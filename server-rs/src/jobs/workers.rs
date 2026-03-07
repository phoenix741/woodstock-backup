//! Exécuteurs/Workers pour les trois files (métriques retirées provisoirement)

use std::sync::Arc;

use apalis::prelude::{Attempt, Data, Storage, TaskId};
// plus d'import apalis Error ici, uniquement eyre::Result dans les handlers
use eyre::{eyre, Result};
use tracing::instrument;

use super::progress::{JobKind, ProgressPublisher, ProgressUpdate};
use crate::jobs::state::ApiWorkerState;
use crate::jobs::types::*;
use chrono::Local;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn, Instrument};
use woodstock::config::{Context, DEFAULT_CHANNEL_BUFFER_SIZE};
use woodstock::server::backup::remove_machine::RemoveBackupMachine;
use woodstock::server::backup::remove_state::RemoveState;
use woodstock::server::backup::restore_machine::{RestoreBackupMachine, ShareSelection};
use woodstock::server::backup::restore_state::RestoreState;
use woodstock::server::backup::save_machine::SaveBackupMachine;
use woodstock::server::backup::save_state::{BackupExecutionState, BackupState};
use woodstock::server::client::grpc::BackupGrpcClient;
use woodstock::server::pool::fsck_machine::FsckMachine;
use woodstock::server::pool::fsck_state::FsckState;
use woodstock::server::pool::pool_cleaner_machine::PoolCleanerMachine;
use woodstock::server::pool::pool_cleaner_state::CleanerState;
use woodstock::server::progression::BackupProgression;
use woodstock::statistics::{disk_stats::append_disk_history, instant_stats::get_space};
use woodstock::utils::lock_redis::{PoolLockRedis, LOCK_TTL};

#[derive(Clone)]
pub struct JobExecutors {
    pub progress: Option<ProgressPublisher>,
}

impl JobExecutors {
    pub fn new() -> Self {
        Self { progress: None }
    }
    pub fn with_progress(mut self, progress: ProgressPublisher) -> Self {
        self.progress = Some(progress);
        self
    }

    /// Publie un état "Skipped" dans Redis pour un job de backup ignoré,
    /// afin que le frontend affiche correctement l'état final du job.
    async fn publish_skipped_backup(&self, task_id: &TaskId, host: &str) {
        if let Some(publi) = &self.progress {
            let skipped_state = BackupState {
                execution_state: BackupExecutionState::Skipped,
                error_state: None,
                global_progression: BackupProgression::default(),
                pre_command_states: std::collections::HashMap::new(),
                share_states: std::collections::HashMap::new(),
                post_command_states: std::collections::HashMap::new(),
            };
            if let Err(e) = publi
                .update_progress(&task_id.to_string(), ProgressUpdate::Backup(skipped_state))
                .await
            {
                warn!(
                    "[{}] Failed to publish skipped state for host {}: {}",
                    task_id, host, e
                );
            }
        }
    }

    #[instrument(skip_all, fields(job_type="backup", host=%job.host, backup_id=%job.id, task_id=%task_id))]
    pub async fn handle_backup(
        &self,
        task_id: TaskId,
        state: Data<Arc<ApiWorkerState>>,
        mut job: BackupJobData,
        attempt: Attempt,
    ) -> Result<()> {
        info!(
            "[{}] Starting backup job for host: {} (attempt: {})",
            task_id,
            job.host,
            attempt.current()
        );
        let context = Context::default();

        let host = job.host.clone();

        // Acquire exclusive lock on the host for the backup operation.
        // We wait up to LOCK_TTL (30 s) — the TTL of the lock key itself — so that if the
        // previous worker died, its lock will have expired before we give up.
        let redis_url = state.config.redis_url();
        let lock = PoolLockRedis::new(&redis_url, &host, "backup")
            .await?
            .try_lock_exclusive_wait(std::time::Duration::from_secs(LOCK_TTL))
            .await?;
        let lock = match lock {
            Some(l) => l,
            None => {
                warn!(
                    "[{}] Skip backup for host {}: lock still held after {}s — another backup is running",
                    task_id, host, LOCK_TTL
                );
                self.publish_skipped_backup(&task_id, &host).await;
                return Ok(());
            }
        };
        info!("[{}] Acquired lock for host: {}", task_id, host);
        // Clone the cancellation token so we can detect lock expiry during the backup.
        // If the host suspends and the Redis TTL expires, the heartbeat fires the token
        // and the backup fails fast → Apalis retries → has_resumable_backup == true → resume.
        let backup_cancel_token = lock.cancellation_token().clone();
        let _lock = lock; // keep alive for scope duration

        // Safety check: if the previous backup number is still InProgress (e.g. the worker
        // crashed mid-backup), the new higher-numbered backup must not run — it would be
        // based on an incomplete reference backup, corrupting the deduplication chain.
        // Instead, we queue a rescue job to resume the interrupted backup and skip this one.
        if let Some(prev_id) = job.previous_id {
            if let Some(prev_backup) = state.backups.get_backup(&host, prev_id).await {
                if prev_backup.status.is_resumable() {
                    warn!(
                        "[{}] Previous backup #{} for {} is still {:?} — queueing rescue job and skipping #{}",
                        task_id, prev_backup.number, host, prev_backup.status, job.number
                    );

                    // Enqueue a rescue job for the interrupted backup.
                    // force=true bypasses the scheduling policy so it runs immediately.
                    let rescue_data = BackupJobData {
                        host: host.clone(),
                        config: job.config.clone(),
                        id: prev_backup.id,
                        number: prev_backup.number,
                        previous_id: None,
                        ip: None,
                        start_date: None,
                        force: true,
                    };
                    let mut storage = state.apalis_redis_storage.backup_storage.clone();
                    let rescue_number = rescue_data.number;
                    match storage
                        .push(BackupQueueJob::Save(rescue_data.clone()))
                        .await
                    {
                        Ok(parts) => {
                            if let Some(publi) = &self.progress {
                                let _ = publi
                                    .create_job(
                                        &parts.task_id.to_string(),
                                        JobKind::with_backup(rescue_data),
                                        Some(&host),
                                    )
                                    .await;
                            }
                            info!(
                                "[{}] Rescue job {} queued for backup #{} of {}",
                                task_id, parts.task_id, rescue_number, host
                            );
                        }
                        Err(e) => {
                            warn!(
                                "[{}] Failed to enqueue rescue job for #{}: {}",
                                task_id, rescue_number, e
                            );
                        }
                    }

                    self.publish_skipped_backup(&task_id, &host).await;
                    return Ok(());
                }
            }
        }

        // Politique: faut-il lancer ce backup maintenant ?
        // A resumable backup (InProgress/Finishing/Aborting/Failed) always runs regardless of
        // policy — it was already started and must be finalized to keep the pool consistent.
        let number = job.number;
        let id = job.id;

        let has_resumable_backup = state
            .backups
            .get_backup(&host, id)
            .await
            .is_some_and(|b| b.status.is_resumable());

        let force = job.force || has_resumable_backup;
        if !force && !state.job_utility.should_backup_host(&host, force).await? {
            warn!(
                "[{}] Skip backup for host {}: policy says not now (force={})",
                task_id, host, force
            );
            self.publish_skipped_backup(&task_id, &host).await;
            return Ok(());
        }
        if has_resumable_backup {
            info!(
                "[{}] Resuming backup #{} for {}: bypassing policy check",
                task_id, number, host
            );
        } else {
            info!("[{}] Policy check passed for host: {}", task_id, host);
        }

        if !has_resumable_backup {
            if !state.job_utility.host_available(&host).await? {
                warn!(
                    "[{}] Skip backup for host {}: host not reachable",
                    task_id, host
                );
                self.publish_skipped_backup(&task_id, &host).await;
                return Ok(());
            }
            info!("[{}] Host is reachable: {}", task_id, host);

            // Résolution IP (définitive) si absente (réutilise même logique que host_available)
            if job.ip.is_none() {
                job.ip = state.job_utility.ping_from_config(&host, &job.config).await;
                if job.ip.is_none() {
                    return Err(eyre!("Impossible de déterminer l'IP atteignable"));
                }
            }
        } else {
            if job.ip.is_none() {
                job.ip = "127.0.0.1:1".parse().ok();
            }
        }

        // SAFETY: job.ip is Some — the is_none() check above returns Err if still None
        let ip = job.ip.clone().unwrap();

        // Set the start date of the job
        if job.start_date.is_none() {
            job.start_date = Some(Local::now());
        }

        // Persist ip and start_date back to the Redis progress snapshot so they
        // appear in the GraphQL API while the backup is running.
        if let Some(publi) = &self.progress {
            if let Err(e) = publi
                .update_backup_job_data(&task_id.to_string(), &job)
                .await
            {
                warn!(
                    "[{}] Failed to update backup job data (ip/start_date) for host {}: {}",
                    task_id, host, e
                );
            }
        }

        // Canal progression
        let (tx, mut rx) = mpsc::channel::<BackupState>(DEFAULT_CHANNEL_BUFFER_SIZE);

        // Thread progression (ici on pourrait mettre à jour métriques; pour l'instant log succinct)
        let publisher = self.progress.clone();
        let job_id_for_task = task_id.clone();

        // Progress reporting task
        let progress_task = tokio::spawn(
            async move {
                while let Some(state) = rx.recv().await {
                    if let Some(publi) = &publisher {
                        if let Err(e) = publi
                            .update_progress(
                                &job_id_for_task.to_string(),
                                ProgressUpdate::Backup(state),
                            )
                            .await
                        {
                            error!(
                                "[{}] Failed to publish backup progress: {}",
                                job_id_for_task, e
                            );
                        }
                    }
                }
            }
            .in_current_span(),
        );

        // Exécution machine
        let grpc_client = BackupGrpcClient::new(&host, &ip, state.config.clone()).await?;

        let mut machine = SaveBackupMachine::new(
            grpc_client,
            &host,
            id,
            number,
            job.previous_id,
            &context,
            Some(tx),
            state.config.clone(),
            state.backups.clone(),
            state.hosts.clone(),
        )
        .await?;
        let exec_res = tokio::select! {
            biased;
            res = machine.execute() => res,
            _ = backup_cancel_token.cancelled() => {
                warn!(
                    "[{}] Backup aborted: Redis lock lost for host {} (host likely suspended)",
                    task_id, host
                );
                Err(eyre!("Backup aborted: Redis lock lost (host likely suspended — will resume on retry)"))
            }
        };

        drop(machine);
        let _ = progress_task.await; // attendre les derniers états

        if let Ok(state) = &exec_res {
            if let Some(publi) = &self.progress {
                if let Err(e) = publi
                    .update_progress(&task_id.to_string(), ProgressUpdate::Backup(state.clone()))
                    .await
                {
                    error!("[{}] Failed to publish final backup state: {}", task_id, e);
                }
            }
        }

        // Gestion de l'erreur pour retry et persistance
        match exec_res {
            Ok(_) => {
                // Enforce retention policy: remove old backups according to the host policy.
                self.enforce_retention_policy(&task_id, &host, id, &state)
                    .await;
                Ok(())
            }
            Err(e) => {
                let error_message = format!("Backup failed: {}", e);
                error!(
                    "[{}] {} (attempt: {})",
                    task_id,
                    error_message,
                    attempt.current()
                );

                // Persister l'erreur dans le manifest
                if let Err(persist_err) = state
                    .backups
                    .update_backup(&host, id, |backup| {
                        backup.error_message = Some(error_message.clone());
                    })
                    .await
                {
                    warn!(
                        "[{}] Failed to persist error message: {}",
                        task_id, persist_err
                    );
                }

                Err(e)
            }
        }
    }

    #[instrument(skip_all, fields(job_type="restore", host=%job.host, backup_id=%job.id, task_id=%task_id))]
    pub async fn handle_restore(
        &self,
        task_id: TaskId,
        state: Data<Arc<ApiWorkerState>>,
        mut job: RestoreJobData,
        attempt: Attempt,
    ) -> Result<()> {
        info!(
            "[{}] Starting restore job for host: {} (attempt: {})",
            task_id,
            job.host,
            attempt.current()
        );
        let context = Context::default();

        let host = job.host.clone();

        // Acquire shared lock on the host for the restore operation
        // Multiple restores can run concurrently
        let redis_url = state.config.redis_url();
        let lock = PoolLockRedis::new(&redis_url, &host, "restore")
            .await?
            .lock_shared()
            .await?;
        let restore_cancel_token = lock.cancellation_token().clone();
        let _lock = lock;

        // Check if the host is available
        if !state.job_utility.host_available(&host).await? {
            debug!("Skip backup for host {}: host not reachable", host);
            return Ok(());
        }

        // Get host configuration
        let host_conf = state.hosts.get_host(&host).await?;
        job.config = Some(host_conf.clone());

        // Resolve IP
        if job.ip.is_none() {
            job.ip = state
                .job_utility
                .ping_from_config(&job.host, &host_conf)
                .await;
            if job.ip.is_none() {
                return Err(eyre!("Impossible de déterminer l'IP atteignable"));
            }
        }
        // SAFETY: job.ip is Some — the is_none() check above returns Err if still None
        let ip = job.ip.clone().unwrap();

        // Set the start date of the job
        if job.start_date.is_none() {
            job.start_date = Some(Local::now());
        }

        // Préparer sélections
        let selections: Vec<ShareSelection<&str, &str>> = job
            .files
            .iter()
            .map(|f| ShareSelection {
                share: f.share.as_str(),
                selection: f.selection.iter().map(|s| s.as_str()).collect(),
            })
            .collect();
        let id = job.id;

        let (tx, mut rx) = mpsc::channel::<RestoreState>(DEFAULT_CHANNEL_BUFFER_SIZE);

        let publisher = self.progress.clone();
        let job_id_for_task = task_id.clone();
        let progress_task = tokio::spawn(
            async move {
                while let Some(state) = rx.recv().await {
                    if let Some(publi) = &publisher {
                        if let Err(e) = publi
                            .update_progress(
                                &job_id_for_task.to_string(),
                                ProgressUpdate::Restore(state),
                            )
                            .await
                        {
                            error!(
                                "[{}] Failed to publish restore progress: {}",
                                job_id_for_task, e
                            );
                        }
                    }
                }
            }
            .in_current_span(),
        );

        // Started event géré par ProgressLayer

        let grpc_client = BackupGrpcClient::new(&job.host, &ip, state.config.clone()).await?;
        let mut machine = RestoreBackupMachine::new(
            grpc_client,
            &job.host,
            id,
            &context,
            Some(tx),
            state.config.clone(),
            state.hosts.clone(),
            state.backups.clone(),
        )
        .await?;
        let exec_res = tokio::select! {
            biased;
            res = machine.execute(&job.destination_directory, &selections) => res,
            _ = restore_cancel_token.cancelled() => {
                warn!(
                    "[{}] Restore aborted: Redis lock lost for host {} (host likely suspended)",
                    task_id, host
                );
                Err(eyre!("Restore aborted: Redis lock lost (host likely suspended)"))
            }
        };

        drop(machine);
        let _ = progress_task.await;

        match exec_res {
            Ok(state) => {
                if let Some(publi) = &self.progress {
                    if let Err(e) = publi
                        .update_progress(&task_id.to_string(), ProgressUpdate::Restore(state))
                        .await
                    {
                        error!("[{}] Failed to publish final restore state: {}", task_id, e);
                    }
                    // Completed event géré par ProgressLayer
                }
            }
            Err(e) => {
                // Failed event géré par ProgressLayer
                return Err(eyre!(e.to_string()));
            }
        }

        Ok(())
    }

    #[instrument(skip_all, fields(job_type="refcnt", task_id=%task_id))]
    pub async fn handle_cleanup_refcnt(
        &self,
        task_id: TaskId,
        state: Data<Arc<ApiWorkerState>>,
        job: CleanupRefcntJobData,
        attempt: Attempt,
    ) -> Result<()> {
        info!(
            "[{}] Starting cleanup refcnt job (attempt: {})",
            task_id,
            attempt.current()
        );
        let context = Context::default();

        let (tx, mut rx) = mpsc::channel::<CleanerState>(DEFAULT_CHANNEL_BUFFER_SIZE);
        let publisher = self.progress.clone();
        let job_id_for_task = task_id.clone();
        let progress_task = tokio::spawn(
            async move {
                while let Some(state) = rx.recv().await {
                    if let Some(publi) = &publisher {
                        if let Err(e) = publi
                            .update_progress(
                                &job_id_for_task.to_string(),
                                ProgressUpdate::CleanupRefcnt(state),
                            )
                            .await
                        {
                            error!(
                                "[{}] Failed to publish cleanup progress: {}",
                                job_id_for_task, e
                            );
                        }
                    }
                }
            }
            .in_current_span(),
        );
        // Started event géré par ProgressLayer

        let machine = PoolCleanerMachine::new(
            job.target.clone().map(std::path::PathBuf::from),
            context.source,
            Some(tx),
            state.config.clone(),
        );
        let exec_res = machine.execute().await;

        drop(machine);
        let _ = progress_task.await;

        match exec_res {
            Err(e) => {
                // Failed event géré par ProgressLayer
                return Err(eyre!(e.to_string()));
            }
            Ok(state) => {
                if let Some(publi) = &self.progress {
                    if let Err(e) = publi
                        .update_progress(&task_id.to_string(), ProgressUpdate::CleanupRefcnt(state))
                        .await
                    {
                        error!("[{}] Failed to publish final cleanup state: {}", task_id, e);
                    }
                }
            }
        }

        Ok(())
    }

    #[instrument(skip_all, fields(job_type="fsck", task_id=%task_id))]
    pub async fn handle_fsck(
        &self,
        task_id: TaskId,
        state: Data<Arc<ApiWorkerState>>,
        job: FsckJobData,
        attempt: Attempt,
    ) -> Result<()> {
        info!(
            "[{}] Starting fsck job (attempt: {})",
            task_id,
            attempt.current()
        );
        let context = Context::default();
        let (tx, mut rx) = mpsc::channel::<FsckState>(DEFAULT_CHANNEL_BUFFER_SIZE);
        let dry = job.dry_run;
        let verify_chunks = job.verify_chunks;
        let publisher = self.progress.clone();
        let job_id_for_task = task_id.clone();

        debug!("Starting fsck process...");

        let progress_task = tokio::spawn(
            async move {
                while let Some(state) = rx.recv().await {
                    if let Some(publi) = &publisher {
                        if let Err(e) = publi
                            .update_progress(
                                &job_id_for_task.to_string(),
                                ProgressUpdate::Fsck(state),
                            )
                            .await
                        {
                            error!(
                                "[{}] Failed to publish fsck progress: {}",
                                job_id_for_task, e
                            );
                        }
                    }
                }
            }
            .in_current_span(),
        );

        debug!("Send fsck process started event...");

        // Started event géré par ProgressLayer

        debug!("Execute ...");

        let machine = FsckMachine::new(
            context.source,
            dry,
            verify_chunks,
            false,
            Some(tx),
            state.config.clone(),
            state.hosts.clone(),
            state.backups.clone(),
        );
        let exec_res = machine.execute().await;

        debug!("Waiting progress ...");

        drop(machine);

        let _ = progress_task.await;

        debug!("Progress task completed ...");

        match exec_res {
            Err(e) => {
                error!("Failed fsck process: {e:?}");
                // Failed event géré par ProgressLayer
                return Err(eyre!(e.to_string()));
            }
            Ok(result) => {
                if let Some(publi) = &self.progress {
                    debug!("Send fsck process completed event...");
                    if let Err(e) = publi
                        .update_progress(&task_id.to_string(), ProgressUpdate::Fsck(result))
                        .await
                    {
                        error!("[{}] Failed to publish final fsck state: {}", task_id, e);
                    }
                }
            }
        }

        info!("End !!!!");

        Ok(())
    }

    #[instrument(skip_all, fields(job_type="remove", host=%job.host, backup_id=%job.id, task_id=%task_id))]
    pub async fn handle_remove(
        &self,
        task_id: TaskId,
        state: Data<Arc<ApiWorkerState>>,
        job: RemoveJobData,
        attempt: Attempt,
    ) -> Result<()> {
        info!(
            "[{}] Starting remove job (attempt: {})",
            task_id,
            attempt.current()
        );
        let context = Context::default();

        let host = job.host.clone();
        let id = job.id;

        // Acquire exclusive lock on the host for the remove operation
        let redis_url = state.config.redis_url();
        let lock = PoolLockRedis::new(&redis_url, &host, "remove")
            .await?
            .lock_exclusive()
            .await?;
        let remove_cancel_token = lock.cancellation_token().clone();
        let _lock = lock;

        let (tx, mut rx) = mpsc::channel::<RemoveState>(DEFAULT_CHANNEL_BUFFER_SIZE);
        let publisher = self.progress.clone();
        let job_id_for_task = task_id.clone();
        let progress_task = tokio::spawn(
            async move {
                while let Some(state) = rx.recv().await {
                    if let Some(publi) = &publisher {
                        if let Err(e) = publi
                            .update_progress(
                                &job_id_for_task.to_string(),
                                ProgressUpdate::Remove(state),
                            )
                            .await
                        {
                            error!(
                                "[{}] Failed to publish remove progress: {}",
                                job_id_for_task, e
                            );
                        }
                    }
                }
            }
            .in_current_span(),
        );
        // Started event géré par ProgressLayer

        let mut machine = RemoveBackupMachine::new(
            &job.host,
            id,
            &context,
            Some(tx),
            state.config.clone(),
            state.backups.clone(),
        );
        let exec_res = tokio::select! {
            biased;
            res = machine.execute() => res,
            _ = remove_cancel_token.cancelled() => {
                warn!(
                    "[{}] Remove aborted: Redis lock lost for host {} (host likely suspended)",
                    task_id, host
                );
                Err(eyre!("Remove aborted: Redis lock lost (host likely suspended)"))
            }
        };

        drop(machine);

        let _ = progress_task.await;

        match exec_res {
            Ok(state) => {
                if let Some(publi) = &self.progress {
                    if let Err(e) = publi
                        .update_progress(&task_id.to_string(), ProgressUpdate::Remove(state))
                        .await
                    {
                        error!("[{}] Failed to publish final remove state: {}", task_id, e);
                    }
                }
            }
            Err(e) => {
                // Failed event géré par ProgressLayer
                return Err(eyre!(e.to_string()));
            }
        }

        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn handle_stats(
        &self,
        state: Data<Arc<ApiWorkerState>>,
        attempt: Attempt,
    ) -> Result<()> {
        info!("Starting stats job (attempt: {})", attempt.current());
        match get_space(&state.config.path.pool_path) {
            Ok(usage) => {
                let _ = append_disk_history(&state.config.path.pool_path, &usage).await;
            }
            Err(e) => {
                tracing::error!("Failed to get disk space: {}", e);
            }
        }
        Ok(())
    }

    /// Enforce retention policy after a successful backup.
    ///
    /// Loads all backups for the host, computes which ones should be deleted
    /// according to the configured [`ScheduledBackupToKeep`] policy, and
    /// enqueues a `Remove` job for each surplus backup.
    ///
    /// The backup that was just created (`current_backup_id`) is always
    /// excluded from deletion as a safety guard.
    async fn enforce_retention_policy(
        &self,
        task_id: &TaskId,
        host: &str,
        current_backup_id: uuid::Uuid,
        state: &ApiWorkerState,
    ) {
        use woodstock::server::backup::retention::get_backups_to_delete;

        let backups = state.backups.get_backups(host).await;

        // Only enforce if the completed backup is actually Completed.
        let is_completed = backups
            .iter()
            .find(|b| b.id == current_backup_id)
            .is_some_and(|b| matches!(b.status, woodstock::config::BackupStatus::Completed));

        if !is_completed {
            return;
        }

        let policy = match state.hosts.get_schedule(host).await {
            Ok(s) => s.backup_to_keep,
            Err(e) => {
                warn!(
                    "[{}] Failed to load retention policy for {}: {}",
                    task_id, host, e
                );
                return;
            }
        };

        let Some(policy) = policy else {
            return;
        };

        let now = Local::now();
        let mut to_delete = get_backups_to_delete(&backups, &policy, now);

        // Never delete the backup we just created (paranoia check).
        to_delete.retain(|&id| id != current_backup_id);

        if to_delete.is_empty() {
            return;
        }

        info!(
            "[{}] Retention: enqueuing {} backup(s) for removal on {}",
            task_id,
            to_delete.len(),
            host
        );

        let count = crate::jobs::producers::enqueue_retention_removals(
            host,
            &backups,
            to_delete,
            &mut state.apalis_redis_storage.backup_storage.clone(),
            self.progress.as_ref(),
        )
        .await;

        info!(
            "[{}] Retention: {} backup(s) successfully enqueued for removal on {}",
            task_id, count, host
        );
    }
}
