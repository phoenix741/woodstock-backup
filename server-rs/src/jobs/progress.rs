//! Job progress publication via Redis (Pub/Sub + hash snapshot)
//! Inspired by the BullMQ `job.progress()` API but externalized in Redis.

use crate::jobs::types::{
    BackupJobData, CleanupRefcntJobData, FsckJobData, RemoveJobData, RestoreJobData, StatsJobData,
};
use apalis::prelude::TaskId;
use eyre::Result;
use futures::{Stream, StreamExt};
use redis::{aio::ConnectionManager, AsyncCommands, AsyncIter, Client as RedisClient};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    pin::Pin,
    sync::Arc,
    task::{Context as TaskContext, Poll},
};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use woodstock::server::{
    backup::{remove_state::RemoveState, restore_state::RestoreState, save_state::BackupState},
    pool::{fsck_state::FsckState, pool_cleaner_state::CleanerState},
};
use woodstock::utils::timed_hashmap::TimedHashMap;

pub const PROGRESS_CHANNEL: &str = "jobs:progress";
pub const PROGRESS_HASH_PREFIX: &str = "job:progress:";

// Max buffer size of 100 messages. If the consumer is slow, old messages are dropped
// to prevent unbounded memory growth.
const MAX_BUFFER_SIZE: usize = 100;
const PROGRESS_THROTTLE_MS: i64 = 1000;

/// TTL (seconds) for snapshots of active or pending jobs.
/// Prevents orphaned CREATED entries (job crashed before being dequeued)
/// from lingering in Redis indefinitely and polluting the "Pending" list.
const PROGRESS_TTL_ACTIVE_SECS: u64 = 86_400; // 24 h

/// TTL (seconds) for snapshots of completed or failed jobs.
/// Keeps the entry visible in the history briefly, then auto-cleaned.
const PROGRESS_TTL_DONE_SECS: u64 = 86_400; // 24 h (cohérent avec ACTIVE)

// Global counter to identify each stream
static STREAM_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Stream with limited buffer that drops old messages if the consumer is slow
struct BufferedProgressStream {
    inner: Pin<Box<dyn Stream<Item = ProgressEvent> + Send>>,
    buffer: VecDeque<ProgressEvent>,
    max_buffer_size: usize,
    dropped_count: usize,
    stream_id: u64,
    poll_count: usize,
}

impl BufferedProgressStream {
    fn new(
        inner: Pin<Box<dyn Stream<Item = ProgressEvent> + Send>>,
        max_buffer_size: usize,
    ) -> Self {
        let stream_id = STREAM_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        info!(
            "[Stream #{}] Creating BufferedProgressStream with max buffer size {}",
            stream_id, max_buffer_size
        );
        Self {
            inner,
            buffer: VecDeque::with_capacity(max_buffer_size),
            max_buffer_size,
            dropped_count: 0,
            stream_id,
            poll_count: 0,
        }
    }
}

impl Drop for BufferedProgressStream {
    fn drop(&mut self) {
        info!(
            "[Stream #{}] BufferedProgressStream dropped after {} polls, total dropped messages: {}",
            self.stream_id, self.poll_count, self.dropped_count
        );
    }
}

impl Stream for BufferedProgressStream {
    type Item = ProgressEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
        self.poll_count += 1;

        // Try to fill the buffer from the source stream
        loop {
            match self.inner.as_mut().poll_next(cx) {
                Poll::Ready(Some(event)) => {
                    // If the buffer is full, remove the oldest message
                    if self.buffer.len() >= self.max_buffer_size {
                        self.buffer.pop_front();
                        self.dropped_count += 1;

                        if self.dropped_count % 10 == 0 {
                            warn!(
                                "[Stream #{}] {} messages dropped due to slow consumer",
                                self.stream_id, self.dropped_count
                            );
                        }
                    }
                    self.buffer.push_back(event);
                }
                Poll::Ready(None) => {
                    // Source stream finished
                    break;
                }
                Poll::Pending => {
                    // No more messages available for now
                    break;
                }
            }
        }

        // Return the oldest message from the buffer
        if let Some(event) = self.buffer.pop_front() {
            Poll::Ready(Some(event))
        } else {
            // Buffer empty, check if source stream is finished
            match self.inner.as_mut().poll_next(cx) {
                Poll::Ready(None) => Poll::Ready(None),
                _ => Poll::Pending,
            }
        }
    }
}

/// Enum for progress updates only
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressUpdate {
    Backup(BackupState),
    Restore(RestoreState),
    Remove(RemoveState),
    CleanupRefcnt(CleanerState),
    Fsck(FsckState),
    Stats(()), // Stats n'ont pas de progression
}

/// Simple filter applied to snapshots and events
#[derive(Debug, Clone, Default)]
pub struct ProgressFilter {
    pub job_id: Option<String>,
    pub kind: Option<String>,
    pub host: Option<String>,
    pub status: Option<JobStatus>,
}

impl ProgressFilter {
    fn matches(&self, json: &ProgressEvent) -> bool {
        if let Some(job_id) = &self.job_id {
            if json.job_id != *job_id {
                return false;
            }
        }
        if let Some(kind) = &self.kind {
            if json.kind.as_str() != kind.as_str() {
                return false;
            }
        }
        if let Some(host) = &self.host {
            if json.host.as_deref() != Some(host.as_str()) {
                return false;
            }
        }
        if let Some(status) = &self.status {
            if json.status != *status {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Created,
    Started,
    Completed,
    Failed,
}

impl JobStatus {
    pub fn to_string(&self) -> Result<String> {
        let json = serde_json::to_string(self)?;

        Ok(json)
    }

    pub fn from_str(s: &str) -> Result<Self> {
        let status: JobStatus = serde_json::from_str(s)?;
        Ok(status)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "D: Serialize + DeserializeOwned, P: Serialize + DeserializeOwned")]
pub struct ProgressData<D, P>
where
    D: Serialize + DeserializeOwned,
    P: Serialize + DeserializeOwned,
{
    pub data: D,
    pub progress: Option<P>,
}

impl<D, P> ProgressData<D, P>
where
    D: Serialize + DeserializeOwned,
    P: Serialize + DeserializeOwned,
{
    pub fn with_progress(mut self, progress: P) -> Self {
        self.progress = Some(progress);
        self
    }

    pub fn to_string(&self) -> Result<String> {
        let json = serde_json::to_string(self)?;
        Ok(json)
    }

    pub fn from_str(s: &str) -> Result<Self> {
        let pd: ProgressData<D, P> = serde_json::from_str(s)?;
        Ok(pd)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    Backup(ProgressData<BackupJobData, BackupState>),
    Restore(ProgressData<RestoreJobData, RestoreState>),
    Remove(ProgressData<RemoveJobData, RemoveState>),
    CleanupRefcnt(ProgressData<CleanupRefcntJobData, CleanerState>),
    Fsck(ProgressData<FsckJobData, FsckState>),
    Stats(ProgressData<StatsJobData, ()>),
}

impl JobKind {
    pub fn as_str(&self) -> &str {
        match self {
            JobKind::Backup(_) => "backup",
            JobKind::Restore(_) => "restore",
            JobKind::Remove(_) => "remove",
            JobKind::CleanupRefcnt(_) => "cleanup_refcnt",
            JobKind::Fsck(_) => "fsck",
            JobKind::Stats(_) => "stats",
        }
    }

    /// Met à jour uniquement la progression, en gardant les mêmes données
    pub fn with_progress_update(self, progress_update: ProgressUpdate) -> Result<Self> {
        match (self, progress_update) {
            (JobKind::Backup(mut pd), ProgressUpdate::Backup(progress)) => {
                pd.progress = Some(progress);
                Ok(JobKind::Backup(pd))
            }
            (JobKind::Restore(mut pd), ProgressUpdate::Restore(progress)) => {
                pd.progress = Some(progress);
                Ok(JobKind::Restore(pd))
            }
            (JobKind::Remove(mut pd), ProgressUpdate::Remove(progress)) => {
                pd.progress = Some(progress);
                Ok(JobKind::Remove(pd))
            }
            (JobKind::CleanupRefcnt(mut pd), ProgressUpdate::CleanupRefcnt(progress)) => {
                pd.progress = Some(progress);
                Ok(JobKind::CleanupRefcnt(pd))
            }
            (JobKind::Fsck(mut pd), ProgressUpdate::Fsck(progress)) => {
                pd.progress = Some(progress);
                Ok(JobKind::Fsck(pd))
            }
            (JobKind::Stats(mut pd), ProgressUpdate::Stats(progress)) => {
                pd.progress = Some(progress);
                Ok(JobKind::Stats(pd))
            }
            _ => Err(eyre::eyre!("Progress update type doesn't match job kind")),
        }
    }
}

impl JobKind {
    pub fn with_backup(data: BackupJobData) -> Self {
        JobKind::Backup(ProgressData {
            data,
            progress: None,
        })
    }

    pub fn with_restore(data: RestoreJobData) -> Self {
        JobKind::Restore(ProgressData {
            data,
            progress: None,
        })
    }

    pub fn with_remove(data: RemoveJobData) -> Self {
        JobKind::Remove(ProgressData {
            data,
            progress: None,
        })
    }

    pub fn with_cleanup_refcnt(data: CleanupRefcntJobData) -> Self {
        JobKind::CleanupRefcnt(ProgressData {
            data,
            progress: None,
        })
    }

    pub fn with_fsck(data: FsckJobData) -> Self {
        JobKind::Fsck(ProgressData {
            data,
            progress: None,
        })
    }

    pub fn with_stats(data: StatsJobData) -> Self {
        JobKind::Stats(ProgressData {
            data,
            progress: None,
        })
    }

    pub fn with_backup_progress(self, progress: BackupState) -> Self {
        match self {
            JobKind::Backup(mut pd) => {
                pd.progress = Some(progress);
                JobKind::Backup(pd)
            }
            other => other,
        }
    }

    pub fn with_restore_progress(self, progress: RestoreState) -> Self {
        match self {
            JobKind::Restore(mut pd) => {
                pd.progress = Some(progress);
                JobKind::Restore(pd)
            }
            other => other,
        }
    }

    pub fn with_remove_progress(self, progress: RemoveState) -> Self {
        match self {
            JobKind::Remove(mut pd) => {
                pd.progress = Some(progress);
                JobKind::Remove(pd)
            }
            other => other,
        }
    }

    pub fn with_cleanup_refcnt_progress(self, progress: CleanerState) -> Self {
        match self {
            JobKind::CleanupRefcnt(mut pd) => {
                pd.progress = Some(progress);
                JobKind::CleanupRefcnt(pd)
            }
            other => other,
        }
    }

    pub fn with_fsck_progress(self, progress: FsckState) -> Self {
        match self {
            JobKind::Fsck(mut pd) => {
                pd.progress = Some(progress);
                JobKind::Fsck(pd)
            }
            other => other,
        }
    }

    pub fn create_event(self, job_id: &TaskId, status: JobStatus) -> ProgressEvent {
        ProgressEvent::new(&job_id.to_string(), self, status)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub job_id: String,
    pub kind: JobKind,
    pub status: JobStatus,
    pub timestamp: i64,

    pub host: Option<String>,

    pub failed_reason: Option<String>,
}

impl ProgressEvent {
    pub fn new(job_id: &str, kind: JobKind, status: JobStatus) -> Self {
        Self {
            job_id: job_id.to_string(),
            kind,
            status,
            host: None,
            timestamp: chrono::Local::now().timestamp(),
            failed_reason: None,
        }
    }

    pub fn with_host(mut self, host: &str) -> Self {
        self.host = Some(host.to_string());
        self
    }

    pub fn with_failed_reason(mut self, failed_reason: &str) -> Self {
        self.failed_reason = Some(failed_reason.to_string());
        self
    }

    /// Reconstitue un ProgressEvent depuis les champs Redis sous forme de HashMap
    pub fn from_redis_hashmap(fields: HashMap<String, String>) -> Result<Self> {
        // Nouvelle approche : utiliser le JSON complet s'il existe
        if let Some(event_json) = fields.get("event_json") {
            return serde_json::from_str(event_json)
                .map_err(|e| eyre::eyre!("Failed to deserialize event_json: {}", e));
        }

        // Fallback pour compatibilité avec anciens snapshots (sans event_json)
        let job_id = fields
            .get("job_id")
            .ok_or_else(|| eyre::eyre!("Missing job_id field"))?
            .clone();

        let json = fields
            .get("json")
            .ok_or_else(|| eyre::eyre!("Missing data field"))?;
        let kind = serde_json::from_str::<JobKind>(json)?;

        let status = fields
            .get("status")
            .ok_or_else(|| eyre::eyre!("Missing status field"))?;

        let timestamp = fields
            .get("timestamp")
            .ok_or_else(|| eyre::eyre!("Missing timestamp field"))?
            .parse::<i64>()
            .unwrap_or(0);

        let host = fields.get("host").cloned();

        let failed_reason = fields.get("failed_reason").cloned();

        let status = JobStatus::from_str(status.as_str())?;

        Ok(Self {
            job_id,
            kind,
            status,
            timestamp,
            host,
            failed_reason,
        })
    }

    /// Génère les champs Redis pour hset_multiple
    pub fn to_redis_fields(&self) -> Result<Vec<(&str, String)>> {
        // Sérialiser UNE SEULE fois l'événement complet
        let event_json = serde_json::to_string(self).map_err(|e| {
            eyre::eyre!(
                "Failed to serialize ProgressEvent (job_id={}, kind={}) to JSON: {}",
                self.job_id,
                self.kind.as_str(),
                e
            )
        })?;

        Ok(vec![
            ("job_id", self.job_id.clone()),
            ("kind", self.kind.as_str().to_string()),
            ("status", self.status.to_string()?),
            ("timestamp", self.timestamp.to_string()),
            ("host", self.host.as_deref().unwrap_or("").to_string()),
            (
                "failed_reason",
                self.failed_reason.as_deref().unwrap_or("").to_string(),
            ),
            ("event_json", event_json), // Event complet pour lecture rapide
        ])
    }
}

#[derive(Clone)]
pub struct ProgressPublisher {
    _client: RedisClient,
    conn: Arc<Mutex<ConnectionManager>>,
    last_progress_publish: Arc<Mutex<TimedHashMap<String, i64>>>,
}

impl ProgressPublisher {
    pub async fn new(client: RedisClient) -> Result<Self> {
        // Création d'un ConnectionManager asynchrone (redis-rs 0.32)
        let manager = ConnectionManager::new(client.clone()).await?;
        Ok(Self {
            _client: client,
            conn: Arc::new(Mutex::new(manager)),
            // TTL de 10 minutes - les jobs sont généralement terminés avant
            last_progress_publish: Arc::new(Mutex::new(TimedHashMap::new(
                std::time::Duration::from_secs(600),
            ))),
        })
    }

    fn snapshot_key(job_id: &str) -> String {
        format!("{}{}", PROGRESS_HASH_PREFIX, job_id)
    }

    // Méthode interne de publication (sans throttling)
    async fn publish_internal(&self, ev: &ProgressEvent) -> Result<()> {
        let mut g = self.conn.lock().await;
        let key = Self::snapshot_key(&ev.job_id);

        // Générer les champs (une seule sérialisation dans to_redis_fields)
        let fields = ev.to_redis_fields()?;

        // Extraire le JSON pour le PUBLISH (réutilisation)
        let event_json = fields
            .iter()
            .find(|(k, _)| *k == "event_json")
            .map(|(_, v)| v.clone())
            .ok_or_else(|| eyre::eyre!("event_json not found in fields"))?;

        // Convertir pour Redis
        let field_refs: Vec<(&str, &str)> = fields.iter().map(|(k, v)| (*k, v.as_str())).collect();
        let _: () = g.hset_multiple(&key, &field_refs).await?;

        // Set a TTL on the snapshot to prevent orphaned entries:
        // - CREATED / STARTED : 24 h (job may wait or run for a long time)
        // - COMPLETED / FAILED : 1 h (keep visible in history briefly, then auto-clean)
        let ttl = match ev.status {
            JobStatus::Completed | JobStatus::Failed => PROGRESS_TTL_DONE_SECS,
            _ => PROGRESS_TTL_ACTIVE_SECS,
        };
        let _: () = g.expire(&key, ttl as i64).await?;

        // Réutiliser le JSON déjà sérialisé pour le PUBLISH
        let _: () = g.publish(PROGRESS_CHANNEL, event_json).await?;

        Ok(())
    }

    async fn can_publish_progress(&self, job_id: &str) -> bool {
        let current_time = chrono::Local::now().timestamp_millis();
        let mut last_publish = self.last_progress_publish.lock().await;

        if let Some(&last_time) = last_publish.get(job_id) {
            if current_time - last_time < PROGRESS_THROTTLE_MS {
                return false;
            }
        }

        last_publish.insert(job_id.to_string(), current_time);
        true
    }

    // API publique pour création de job
    pub async fn create_job(&self, job_id: &str, kind: JobKind, host: Option<&str>) -> Result<()> {
        let mut event = ProgressEvent::new(job_id, kind, JobStatus::Created);
        if let Some(host) = host {
            event = event.with_host(host);
        }
        self.publish_internal(&event).await
    }

    /// Met à jour les champs `ip` et `start_date` du `BackupJobData` stocké dans Redis.
    /// Appelé par le worker après résolution de l'IP et initialisation de la date de début.
    pub async fn update_backup_job_data(&self, job_id: &str, data: &BackupJobData) -> Result<()> {
        match self.get_current_state(job_id).await? {
            Some(mut current) => {
                if let JobKind::Backup(ref mut pd) = current.kind {
                    pd.data.ip = data.ip.clone();
                    pd.data.start_date = data.start_date;
                }
                current.timestamp = chrono::Local::now().timestamp();
                self.publish_internal(&current).await
            }
            None => {
                tracing::warn!("Job {} not found during update_backup_job_data", job_id);
                Ok(())
            }
        }
    }

    // Mise à jour de la progression avec throttling (nouvelle version qui ne change que progress)
    pub async fn update_progress(
        &self,
        job_id: &str,
        progress_update: ProgressUpdate,
    ) -> Result<()> {
        // Vérifier le throttling
        if !self.can_publish_progress(job_id).await {
            // Throttlé - on met à jour seulement le snapshot sans publier
            return self
                .update_snapshot_progress_only(job_id, progress_update)
                .await;
        }

        match self.get_current_state(job_id).await? {
            Some(mut current) => {
                current.kind = current.kind.with_progress_update(progress_update)?;
                current.timestamp = chrono::Local::now().timestamp();
                self.publish_internal(&current).await
            }
            None => {
                tracing::warn!(
                        "Job {} not found during update_progress, cannot update progress without initial data",
                    job_id
                );
                Err(eyre::eyre!("Job {} not found for progress update", job_id))
            }
        }
    }

    // Mise à jour snapshot seulement pour la progression (pas de publication)
    async fn update_snapshot_progress_only(
        &self,
        job_id: &str,
        progress_update: ProgressUpdate,
    ) -> Result<()> {
        let event = match self.get_current_state(job_id).await? {
            Some(mut current) => {
                current.kind = current.kind.with_progress_update(progress_update)?;
                current.timestamp = chrono::Local::now().timestamp();
                current
            }
            None => {
                tracing::warn!("Job {} not found during throttled update_progress", job_id);
                return Ok(()); // Ne pas créer automatiquement lors d'un update throttlé
            }
        };

        // Mise à jour du snapshot seulement (réutilise to_redis_fields)
        let mut g = self.conn.lock().await;
        let key = Self::snapshot_key(job_id);

        let fields = event.to_redis_fields()?;
        let field_refs: Vec<(&str, &str)> = fields.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let _: () = g.hset_multiple(&key, &field_refs).await?;
        // Refresh TTL on progress updates — the job is still active
        let _: () = g.expire(&key, PROGRESS_TTL_ACTIVE_SECS as i64).await?;
        Ok(())
    }

    // Mise à jour du statut (toujours publié immédiatement)
    pub async fn update_status(&self, job_id: &str, status: JobStatus) -> Result<()> {
        match self.get_current_state(job_id).await? {
            Some(mut current) => {
                current.failed_reason = None;
                current.status = status;
                current.timestamp = chrono::Local::now().timestamp();
                let result = self.publish_internal(&current).await;

                // Nettoyer le throttling si le job est terminé ou échoué
                if matches!(status, JobStatus::Completed | JobStatus::Failed) {
                    let mut last_publish = self.last_progress_publish.lock().await;
                    last_publish.remove(job_id);
                }

                result
            }
            None => Err(eyre::eyre!("Job {} not found for status update", job_id)),
        }
    }

    // Marquer comme échoué
    pub async fn mark_failed(&self, job_id: &str, reason: &str) -> Result<()> {
        match self.get_current_state(job_id).await? {
            Some(mut current) => {
                current.status = JobStatus::Failed;
                current.failed_reason = Some(reason.to_string());
                current.timestamp = chrono::Local::now().timestamp();
                let result = self.publish_internal(&current).await;

                // Nettoyer le throttling
                let mut last_publish = self.last_progress_publish.lock().await;
                last_publish.remove(job_id);

                result
            }
            None => Err(eyre::eyre!("Job {} not found for failure marking", job_id)),
        }
    }

    // Marquer comme terminé avec succès
    pub async fn mark_completed(&self, job_id: &str) -> Result<()> {
        self.update_status(job_id, JobStatus::Completed).await
    }

    // Méthode helper pour récupérer l'état actuel
    async fn get_current_state(&self, job_id: &str) -> Result<Option<ProgressEvent>> {
        let key = Self::snapshot_key(job_id);
        let mut g = self.conn.lock().await;

        // Essayer d'abord la nouvelle structure avec HashMap
        let fields: Result<std::collections::HashMap<String, String>, _> = g.hgetall(&key).await;

        if let Ok(fields) = fields {
            if !fields.is_empty() && fields.contains_key("job_id") {
                return ProgressEvent::from_redis_hashmap(fields).map(Some);
            }
        }

        // Fallback vers l'ancienne structure avec JSON complet (pour compatibilité)
        let json: String = g.hget(&key, "json").await.ok().unwrap_or_default();
        if json.is_empty() {
            return Ok(None);
        }

        let event: ProgressEvent = serde_json::from_str(&json)?;
        Ok(Some(event))
    }

    // Méthode pour forcer la publication (ignorer le throttling) - nouvelle version
    pub async fn force_publish_progress(
        &self,
        job_id: &str,
        progress_update: ProgressUpdate,
    ) -> Result<()> {
        // Réinitialiser le throttling pour ce job
        {
            let mut last_publish = self.last_progress_publish.lock().await;
            last_publish.remove(job_id);
        }

        // Puis publier normalement
        self.update_progress(job_id, progress_update).await
    }

    /// Nettoie les entrées de throttling expirées (optionnel, car TimedHashMap le fait automatiquement)
    pub async fn cleanup_expired_throttling(&self) {
        let mut last_publish = self.last_progress_publish.lock().await;
        last_publish.cleanup_expired();
    }
}

#[derive(Clone)]
pub struct ProgressReader {
    client: RedisClient,
    conn: Arc<Mutex<ConnectionManager>>,
}

impl ProgressReader {
    pub async fn new(client: RedisClient) -> Result<Self> {
        // Création d'un ConnectionManager asynchrone (redis-rs 0.32)
        let manager = ConnectionManager::new(client.clone()).await?;
        Ok(Self {
            client,
            conn: Arc::new(Mutex::new(manager)),
        })
    }

    fn snapshot_key(job_id: &str) -> String {
        format!("{}{}", PROGRESS_HASH_PREFIX, job_id)
    }

    async fn get_key(&self, key: &str) -> Option<ProgressEvent> {
        let mut g = self.conn.lock().await;

        // Essayer d'abord la nouvelle structure avec HashMap
        let fields: std::collections::HashMap<String, String> = g.hgetall(key).await.ok()?;

        return ProgressEvent::from_redis_hashmap(fields).ok();
    }

    pub async fn get(&self, job_id: &str) -> Option<ProgressEvent> {
        let key = Self::snapshot_key(job_id);
        self.get_key(&key).await
    }

    pub async fn list(&self, filter: ProgressFilter) -> Result<Vec<ProgressEvent>> {
        let mut keys: Vec<String> = Vec::new();
        {
            let mut g = self.conn.lock().await;

            let pattern = format!("{}*", PROGRESS_HASH_PREFIX);
            let mut iter: AsyncIter<'_, _> = g.scan_match(pattern).await?;
            while let Some(k) = iter.next_item().await {
                match k {
                    Ok(k) => keys.push(k),
                    Err(err) => {
                        error!("Redis scan failed: {}", err);
                    }
                }
            }
        }
        debug!("Found {} progress keys in Redis", keys.len());

        let mut out = Vec::with_capacity(keys.len());
        for key in keys {
            if let Some(event) = self.get_key(&key).await {
                debug!(
                    "Found progress event for job_id {}: {:?}, {:?}",
                    event.job_id, event, filter
                );

                if filter.matches(&event) {
                    out.push(event);
                }
            }
        }

        // Sort by timestamp from most recent to old
        out.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(out)
    }

    pub async fn stream(
        &self,
        filter: ProgressFilter,
    ) -> Pin<Box<dyn Stream<Item = ProgressEvent> + Send>> {
        let client = self.client.clone();

        let mut pubsub = match client.get_async_pubsub().await {
            Ok(pubsub) => pubsub,
            Err(e) => {
                error!("Failed to get Redis pubsub: {}", e);
                return Box::pin(futures::stream::empty::<ProgressEvent>());
            }
        };

        if let Err(e) = pubsub.subscribe(PROGRESS_CHANNEL).await {
            error!("Failed to subscribe to progress channel: {}", e);
            return Box::pin(futures::stream::empty::<ProgressEvent>());
        }

        // Wrap filter dans Arc pour éviter de cloner les Strings à chaque message
        let filter = Arc::new(filter);

        // Convert PubSub into a message stream, then filter/map to ProgressEvent
        let base_stream = pubsub.into_on_message().filter_map(move |msg| {
            let filter = Arc::clone(&filter);
            async move {
                match msg.get_payload::<String>() {
                    Ok(payload) => match serde_json::from_str::<ProgressEvent>(&payload) {
                        Ok(event) if filter.matches(&event) => Some(event),
                        _ => None,
                    },
                    Err(_) => None,
                }
            }
        });

        Box::pin(BufferedProgressStream::new(
            Box::pin(base_stream),
            MAX_BUFFER_SIZE,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::collections::HashMap;
    use woodstock::{
        config::{HostConfigOperation, HostConfiguration, DEFAULT_PORT},
        server::backup::save_state::BackupExecutionState,
    };

    // Configuration minimale pour les tests
    fn minimal_host_config() -> HostConfiguration {
        HostConfiguration {
            password: String::new(),
            addresses: None,
            port: DEFAULT_PORT,
            operations: HostConfigOperation {
                pre_commands: None,
                operation: None,
                post_commands: None,
            },
            schedule: None,
        }
    }

    #[test]
    fn test_progress_event_basic_serialization_backup() {
        let data = BackupJobData {
            host: "test-host".to_string(),
            config: minimal_host_config(),
            id: uuid::Uuid::nil(),
            previous_id: None,
            number: 1,
            ip: None,
            start_date: None,
            force: false,
        };

        // Construire un BackupState valide à partir de la configuration minimale
        let mut progress = BackupState::from_configuration(&minimal_host_config());
        let fixed = chrono::Local.timestamp_opt(1_600_000_000, 0).unwrap();

        // Dates globales figées
        progress.global_progression.start_date = fixed;
        progress.global_progression.start_transfer_date = Some(fixed);
        progress.global_progression.end_transfer_date = Some(fixed);

        let kind = JobKind::with_backup(data).with_backup_progress(progress);
        let event = ProgressEvent::new("job123", kind, JobStatus::Started).with_host("test-host");

        // Test de la sérialisation directe (nouvelle approche)
        let event_json = serde_json::to_string(&event).unwrap();
        assert!(event_json.contains("job123"));
        assert!(event_json.contains("test-host"));

        // Test de désérialisation
        let deserialized: ProgressEvent = serde_json::from_str(&event_json).unwrap();
        assert_eq!(deserialized.job_id, "job123");
        assert_eq!(deserialized.status, JobStatus::Started);

        // Test de compatibilité backward avec to_redis_fields
        let fields = event.to_redis_fields().unwrap();

        let field_map: HashMap<&str, String> = fields.into_iter().collect();
        assert_eq!(field_map.get("job_id").unwrap(), "job123");
        assert_eq!(field_map.get("kind").unwrap(), "backup");
        assert_eq!(field_map.get("status").unwrap(), "\"started\"");
        assert_eq!(field_map.get("host").unwrap(), "test-host");

        // Vérifier que le champ event_json contient l'événement complet
        assert!(field_map.get("event_json").is_some());
        let event_json_field = field_map.get("event_json").unwrap();
        assert!(event_json_field.contains("job123"));
        assert!(event_json_field.contains("test-host"));
    }

    #[test]
    fn test_progress_event_basic_deserialization_backup() {
        // Créer un event complet pour générer le JSON
        let data = BackupJobData {
            host: "backup-host".to_string(),
            config: minimal_host_config(),
            id: uuid::Uuid::nil(),
            previous_id: None,
            number: 42,
            ip: None,
            start_date: None,
            force: false,
        };

        let mut progress = BackupState::from_configuration(&minimal_host_config());
        let fixed = chrono::Local.timestamp_opt(1_600_000_000, 0).unwrap();
        progress.global_progression.start_date = fixed;
        progress.global_progression.start_transfer_date = Some(fixed);
        progress.global_progression.end_transfer_date = Some(fixed);

        let kind = JobKind::with_backup(data).with_backup_progress(progress);
        let kind_json = serde_json::to_string(&kind).unwrap();

        // Ancien format (avec champ "json" au lieu de "event_json")
        let mut fields = HashMap::new();
        fields.insert("job_id".to_string(), "job456".to_string());
        fields.insert("kind".to_string(), "backup".to_string());
        fields.insert("json".to_string(), kind_json); // Ancien format
        fields.insert("status".to_string(), "\"started\"".to_string());
        fields.insert("timestamp".to_string(), "1234567890".to_string());
        fields.insert("host".to_string(), "test-host".to_string());
        fields.insert("failed_reason".to_string(), "".to_string());

        let event = ProgressEvent::from_redis_hashmap(fields).unwrap();

        assert_eq!(event.job_id, "job456");
        assert_eq!(event.status, JobStatus::Started);
        assert_eq!(event.timestamp, 1234567890);
        assert_eq!(event.host, Some("test-host".to_string()));
        assert!(matches!(event.kind, JobKind::Backup(_)));

        if let JobKind::Backup(pd) = event.kind {
            assert_eq!(pd.data.host, "backup-host");
            assert_eq!(pd.data.number, 42);
            assert_eq!(
                pd.progress.unwrap().execution_state,
                BackupExecutionState::Waiting
            );
        }
    }
}
