//! Exécuteurs/Workers pour les trois files (métriques retirées provisoirement)

use std::sync::Arc;

use apalis::prelude::{Attempt, Data, Storage, TaskId};
// plus d'import apalis Error ici, uniquement eyre::Result dans les handlers
use eyre::{eyre, Result, WrapErr};
use tracing::instrument;

use super::progress::{
    clear_cancel_request, is_cancel_requested, spawn_cancel_watcher, JobKind, ProgressPublisher,
    ProgressUpdate,
};
use crate::jobs::state::ApiWorkerState;
use crate::jobs::types::*;
use chrono::Local;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn, Instrument};
use woodstock::archiving::{
    ArchiveHostExecutionState, ArchiveHostState, ArchiveProgressCounters, ArchiveState,
};
use woodstock::config::{Context, DEFAULT_CHANNEL_BUFFER_SIZE};
use woodstock::server::backup::remove_machine::RemoveBackupMachine;
use woodstock::server::backup::remove_state::RemoveState;
use woodstock::server::backup::restore_machine::{RestoreBackupMachine, ShareSelection};
use woodstock::server::backup::restore_state::{RestoreExecutionState, RestoreState};
use woodstock::server::backup::save_machine::SaveBackupMachine;
use woodstock::server::backup::save_state::{BackupExecutionState, BackupState};
use woodstock::server::client::grpc::BackupGrpcClient;
use woodstock::server::pool::fsck_machine::FsckMachine;
use woodstock::server::pool::fsck_state::FsckState;
use woodstock::server::pool::pool_cleaner_machine::PoolCleanerMachine;
use woodstock::server::pool::pool_cleaner_state::CleanerState;
use woodstock::server::progression::BackupProgression;
use woodstock::statistics::{disk_stats::append_disk_history, instant_stats::get_space};
use woodstock::utils::lock_redis::{HostLockOperation, LockOperation, PoolLockRedis, LOCK_TTL};

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

    /// Publishes a "cancelled" state for a backup that was cancelled while
    /// still queued — before `SaveBackupMachine` (and therefore any lock,
    /// manifest, or `Backup` entry) was ever created. Unlike `Skipped`,
    /// which means the scheduler will retry later, a cancelled job is a
    /// deliberate stop and is not retried.
    async fn publish_cancelled_backup(&self, task_id: &TaskId, host: &str) {
        if let Some(publi) = &self.progress {
            let cancelled_state = BackupState {
                execution_state: BackupExecutionState::Cancelled,
                error_state: None,
                global_progression: BackupProgression::default(),
                pre_command_states: std::collections::HashMap::new(),
                share_states: std::collections::HashMap::new(),
                post_command_states: std::collections::HashMap::new(),
            };
            if let Err(e) = publi
                .update_progress(
                    &task_id.to_string(),
                    ProgressUpdate::Backup(cancelled_state),
                )
                .await
            {
                warn!(
                    "[{}] Failed to publish cancelled state for host {}: {}",
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

        // A cancel requested while this job was still waiting in the Apalis
        // queue is handled the same way as a running-job cancel: same Redis
        // marker, checked here before any lock/manifest/snapshot exists.
        let redis_client = redis::Client::open(state.config.redis_url())
            .wrap_err("Failed to open Redis client for cancel-marker checks")?;
        if is_cancel_requested(&redis_client, &task_id.to_string())
            .await
            .unwrap_or(false)
        {
            info!(
                "[{}] Backup for host {} was cancelled before it started",
                task_id, host
            );
            let _ = clear_cancel_request(&redis_client, &task_id.to_string()).await;
            self.publish_cancelled_backup(&task_id, &host).await;
            return Ok(());
        }

        // Acquire exclusive lock on the host for the backup operation.
        // We wait up to LOCK_TTL (30 s) — the TTL of the lock key itself — so that if the
        // previous worker died, its lock will have expired before we give up.
        let redis_url = state.config.redis_url();
        let lock = PoolLockRedis::new(
            &redis_url,
            &host,
            LockOperation::Host(HostLockOperation::Backup),
        )
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

        // Re-check blackout at execution time, not just at enqueue time: a job can sit in
        // the queue long enough for a blackout window to start after `decision::try_schedule_host`
        // already let it through.
        if !force {
            if let Some(retry_at) = state.job_utility.is_in_blackout_now(&host).await? {
                warn!(
                    "[{}] Skip backup for host {}: now in blackout until {}",
                    task_id, host, retry_at
                );
                self.publish_skipped_backup(&task_id, &host).await;
                return Ok(());
            }
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

        // Distinct from `backup_cancel_token` above: that one fires on Redis
        // lock loss and is raced against the whole `machine.execute()` future
        // in the `select!` below (bypassing finalization on purpose — there's
        // no pool state left to close cleanly once the lock is gone). This one
        // is a deliberate user cancel and is threaded *into* the machine so it
        // flows through the same finalization pipeline as any other critical
        // error, ending in `BackupStatus::Cancelled` instead of `Aborted`.
        //
        // Spawned right before the machine that consumes its token (rather
        // than right after acquiring the lock above) so none of the early
        // `return`s for the rescue-job/policy/availability checks in between
        // leak a Redis-polling task that would otherwise outlive this job.
        let (user_cancel_token, cancel_watcher) =
            spawn_cancel_watcher(redis_client.clone(), task_id.to_string());

        let mut machine = match SaveBackupMachine::new(
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
            user_cancel_token,
        )
        .await
        {
            Ok(machine) => machine,
            Err(e) => {
                cancel_watcher.finish().await;
                return Err(e);
            }
        };
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
        cancel_watcher.finish().await;
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

        // A cancel requested while this job was still waiting in the Apalis
        // queue — same Redis marker as a running-job cancel, checked before
        // any lock/connection exists.
        let redis_client = redis::Client::open(state.config.redis_url())
            .wrap_err("Failed to open Redis client for cancel-marker checks")?;
        if is_cancel_requested(&redis_client, &task_id.to_string())
            .await
            .unwrap_or(false)
        {
            info!(
                "[{}] Restore for host {} was cancelled before it started",
                task_id, host
            );
            let _ = clear_cancel_request(&redis_client, &task_id.to_string()).await;
            if let Some(publi) = &self.progress {
                let cancelled_state = RestoreState {
                    execution_state: RestoreExecutionState::Cancelled,
                    ..RestoreState::default()
                };
                let _ = publi
                    .update_progress(
                        &task_id.to_string(),
                        ProgressUpdate::Restore(cancelled_state),
                    )
                    .await;
            }
            return Ok(());
        }

        // Acquire shared lock on the host for the restore operation
        // Multiple restores can run concurrently
        let redis_url = state.config.redis_url();
        let lock = PoolLockRedis::new(
            &redis_url,
            &host,
            LockOperation::Host(HostLockOperation::Restore),
        )
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

        // Distinct from `restore_cancel_token` above (Redis lock loss, raced
        // against the whole `machine.execute()` future below). This one is a
        // deliberate user cancel, threaded *into* the machine so it stops at
        // the next file/share boundary and still reaches `close()`.
        //
        // Spawned right before the machine that consumes its token (rather
        // than right after acquiring the lock above) so none of the early
        // `return`s for the availability/IP-resolution checks in between
        // leak a Redis-polling task that would otherwise outlive this job.
        let (user_cancel_token, cancel_watcher) =
            spawn_cancel_watcher(redis_client.clone(), task_id.to_string());

        let mut machine = match RestoreBackupMachine::new(
            grpc_client,
            &job.host,
            id,
            &context,
            Some(tx),
            state.config.clone(),
            state.hosts.clone(),
            state.backups.clone(),
            user_cancel_token,
        )
        .await
        {
            Ok(machine) => machine,
            Err(e) => {
                cancel_watcher.finish().await;
                return Err(e);
            }
        };
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
        cancel_watcher.finish().await;
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

        let redis_client = redis::Client::open(state.config.redis_url())
            .wrap_err("Failed to open Redis client for cancel-marker checks")?;
        if is_cancel_requested(&redis_client, &task_id.to_string())
            .await
            .unwrap_or(false)
        {
            info!("[{}] Fsck job was cancelled before it started", task_id);
            let _ = clear_cancel_request(&redis_client, &task_id.to_string()).await;
            if let Some(publi) = &self.progress {
                let mut cancelled_state = FsckState::new(job.dry_run);
                cancelled_state.cancel();
                if let Err(e) = publi
                    .update_progress(&task_id.to_string(), ProgressUpdate::Fsck(cancelled_state))
                    .await
                {
                    warn!(
                        "[{}] Failed to publish cancelled fsck state: {}",
                        task_id, e
                    );
                }
            }
            return Ok(());
        }

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

        let (fsck_cancel_token, cancel_watcher) =
            spawn_cancel_watcher(redis_client.clone(), task_id.to_string());

        let machine = FsckMachine::new(
            context.source,
            dry,
            verify_chunks,
            false,
            Some(tx),
            state.config.clone(),
            state.hosts.clone(),
            state.backups.clone(),
            fsck_cancel_token,
        );
        let exec_res = machine.execute().await;

        debug!("Waiting progress ...");

        drop(machine);
        cancel_watcher.finish().await;

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
        let lock = PoolLockRedis::new(
            &redis_url,
            &host,
            LockOperation::Host(HostLockOperation::Remove),
        )
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

    /// Archives every host in `job.hostnames` sequentially — one host at a
    /// time, never in parallel — so a profile drives its destination disk
    /// (often removable media) with one write stream instead of seeking
    /// across it from N concurrent jobs.
    ///
    /// A single host failing (no completed backup, lock timeout, write
    /// error) is recorded in the final `ArchiveState::failed_hosts` and
    /// logged, but does not fail the job: this queue's `RetryPolicy` retries
    /// the whole job on `Err`, which would re-archive every host that
    /// already succeeded. Returning `Ok(())` regardless keeps a partial
    /// failure from thrashing the disk with redundant re-archiving of hosts
    /// that were already written successfully.
    #[instrument(skip_all, fields(job_type="archive", profile=%job.profile_name, hosts=job.hostnames.len(), task_id=%task_id))]
    pub async fn handle_archive_run(
        &self,
        task_id: TaskId,
        state: Data<Arc<ApiWorkerState>>,
        job: ArchiveRunJobData,
        attempt: Attempt,
    ) -> Result<()> {
        info!(
            "[{}] Starting archive run for {} host(s) (profile: {}, attempt: {})",
            task_id,
            job.hostnames.len(),
            job.profile_name,
            attempt.current()
        );

        let redis_client = redis::Client::open(state.config.redis_url())
            .wrap_err("Failed to open Redis client for cancel-marker checks")?;
        if is_cancel_requested(&redis_client, &task_id.to_string())
            .await
            .unwrap_or(false)
        {
            info!(
                "[{}] Archive run for profile '{}' was cancelled before it started",
                task_id, job.profile_name
            );
            let _ = clear_cancel_request(&redis_client, &task_id.to_string()).await;
            self.publish_cancelled_archive(&task_id, &job.hostnames)
                .await;
            return Ok(());
        }

        let archiving = woodstock::config::ArchivingConfig::new(state.config.clone());
        let Some(profile) = archiving.get_profile(&job.profile_name).await? else {
            warn!(
                "[{}] Archive profile '{}' no longer exists, skipping",
                task_id, job.profile_name
            );
            return Ok(());
        };

        // Resolve upfront which hosts actually have something to archive,
        // and (for `dir` mode, whose per-host max can't come from
        // `Backup::file_size` — that would count the whole backup rather
        // than just the delta this run writes) size each host before
        // starting, so both the aggregate and per-host progress bars have a
        // known denominator from the first published snapshot.
        let mut resolved = Vec::new();
        for hostname in &job.hostnames {
            match state.backups.get_last_backup(&hostname).await {
                Some(backup) => resolved.push((hostname.clone(), backup)),
                None => info!(
                    "[{}] No completed backup for host '{}', skipping",
                    task_id, hostname
                ),
            }
        }

        let mut resolved_with_max = Vec::with_capacity(resolved.len());
        for (hostname, backup) in resolved {
            let host_max = match profile.format {
                woodstock::config::ArchiveFormat::Tar(_)
                | woodstock::config::ArchiveFormat::TarGz(_)
                | woodstock::config::ArchiveFormat::TarXz(_)
                | woodstock::config::ArchiveFormat::TarZstd(_) => backup.file_size,
                woodstock::config::ArchiveFormat::Dir => {
                    woodstock::archiving::dir_sync::dir_diff_total_size(
                        &state.backups,
                        &hostname,
                        &backup,
                        &profile.destination,
                    )
                    .await
                    .unwrap_or(0)
                }
            };
            resolved_with_max.push((hostname, backup, host_max));
        }

        let hosts_total = resolved_with_max.len();
        let progress_max: u64 = resolved_with_max.iter().map(|(_, _, max)| *max).sum();

        let redis_url = state.config.redis_url();
        let counters = Arc::new(ArchiveProgressCounters::default());

        // Set once, right here — after the sizing loop above (which can
        // itself take real time for `dir` mode's diff) and before the first
        // host actually starts writing — so sizing time never gets counted
        // as transfer time in `ArchiveState::speed`.
        let mut run_state = ArchiveState {
            current_host: None,
            hosts_done: 0,
            hosts_total,
            start_date: Local::now(),
            progress_current: 0,
            progress_max,
            file_count: 0,
            archive_size: 0,
            failed_hosts: Vec::new(),
            cancelled: false,
            host_states: resolved_with_max
                .iter()
                .map(|(hostname, _, host_max)| ArchiveHostState {
                    hostname: hostname.clone(),
                    execution_state: ArchiveHostExecutionState::Waiting,
                    progress_current: 0,
                    progress_max: *host_max,
                    file_count: 0,
                    archive_size: None,
                })
                .collect(),
        };
        self.publish_archive_progress(&task_id, &run_state).await;

        // One token for the whole run, not per host: cancelling stops the
        // host currently writing (tar-family, mid-write — see
        // `archive_one_host`'s doc comment) and skips every host not yet
        // started, exactly like the other two job types.
        let (cancel_token, cancel_watcher) =
            spawn_cancel_watcher(redis_client.clone(), task_id.to_string());

        for (index, (hostname, backup, _host_max)) in resolved_with_max.into_iter().enumerate() {
            run_state.current_host = Some(hostname.clone());
            run_state.host_states[index].execution_state = ArchiveHostExecutionState::InProgress;
            self.publish_archive_progress(&task_id, &run_state).await;

            // Shared lock: reading a host's backups to export them must not
            // run concurrently with a Backup/Restore/Remove (exclusive) on
            // the same host, but multiple archive profiles reading the same
            // host may.
            let lock_result = PoolLockRedis::new(
                &redis_url,
                &hostname,
                LockOperation::Host(HostLockOperation::Archive),
            )
            .await;
            let lock = match lock_result {
                Ok(pool_lock) => {
                    pool_lock
                        .try_lock_shared_wait(std::time::Duration::from_secs(LOCK_TTL))
                        .await
                }
                Err(e) => Err(e),
            };

            let (host_start_bytes, host_start_files) = counters.snapshot();

            let host_result: Result<Option<u64>> = match lock {
                Ok(Some(_lock)) => {
                    let ticker = self.spawn_archive_progress_ticker(
                        &task_id,
                        run_state.clone(),
                        index,
                        host_start_bytes,
                        host_start_files,
                        &counters,
                    );
                    let result = self
                        .archive_one_host(
                            &task_id,
                            &state,
                            &profile,
                            &hostname,
                            &backup,
                            &counters,
                            cancel_token.clone(),
                        )
                        .await;
                    ticker.abort();
                    result
                }
                Ok(None) => {
                    warn!(
                        "[{}] Skip archive for host {}: could not acquire shared lock after {}s",
                        task_id, hostname, LOCK_TTL
                    );
                    Err(eyre!("could not acquire shared lock"))
                }
                Err(e) => Err(e),
            };

            let (total_bytes, total_files) = counters.snapshot();
            run_state.hosts_done += 1;
            run_state.progress_current = total_bytes;
            run_state.file_count = total_files;
            run_state.host_states[index].progress_current = total_bytes - host_start_bytes;
            run_state.host_states[index].file_count = total_files - host_start_files;
            // Tar-family reports its own post-compression file size; `Dir`
            // has none (no single archive file), so its "archive size" is
            // just the bytes this host actually synced to the destination —
            // see the format mapping on `ArchiveHostState::archive_size`.
            run_state.host_states[index].archive_size = match &host_result {
                Ok(Some(size)) => Some(*size),
                Ok(None) => Some(run_state.host_states[index].progress_current),
                Err(_) => None,
            };
            if let Some(size) = run_state.host_states[index].archive_size {
                run_state.archive_size += size;
            }
            let was_cancelled = cancel_token.is_cancelled();
            run_state.host_states[index].execution_state = match &host_result {
                Err(_) if was_cancelled => {
                    // The write error is the cancel itself (see `run_writer`
                    // in tar_writer.rs) — not a real failure, so it's kept
                    // out of `failed_hosts` (which drives a "N host(s)
                    // failed" warning UI treatment).
                    info!(
                        "[{}] Archive cancelled by user while archiving host '{}'",
                        task_id, hostname
                    );
                    ArchiveHostExecutionState::Cancelled
                }
                Err(e) => {
                    warn!(
                        "[{}] Archive failed for host '{}' (profile '{}'): {}",
                        task_id, hostname, job.profile_name, e
                    );
                    run_state.failed_hosts.push(hostname);
                    ArchiveHostExecutionState::Failed
                }
                Ok(_) => ArchiveHostExecutionState::Success,
            };

            self.publish_archive_progress(&task_id, &run_state).await;

            if was_cancelled {
                // Stop the run entirely: hosts not yet started never run at
                // all — only the tar of the host that was in progress (just
                // handled above) is truncated/removed; hosts already
                // Success'd earlier in this loop keep their archive.
                run_state.cancelled = true;
                for remaining in run_state.host_states.iter_mut().skip(index + 1) {
                    remaining.execution_state = ArchiveHostExecutionState::Cancelled;
                }
                self.publish_archive_progress(&task_id, &run_state).await;
                break;
            }
        }

        cancel_watcher.finish().await;

        if !run_state.failed_hosts.is_empty() {
            warn!(
                "[{}] Archive profile '{}' completed with {}/{} host(s) failed: {:?}",
                task_id,
                job.profile_name,
                run_state.failed_hosts.len(),
                hosts_total,
                run_state.failed_hosts
            );
        }

        Ok(())
    }

    /// Archives a single host — tar-family or `dir`, dispatched on
    /// `profile.format`. The caller owns the progress ticker around this
    /// call; `counters` is updated per-entry, synchronously, from inside the
    /// writer itself. Returns the host's final archive size when the format
    /// reports one directly (tar-family, post-compression); `Dir` returns
    /// `None` since there is no single archive file — the caller derives its
    /// archive size from `counters` instead (see call site).
    /// `cancel_token`: consulted mid-write for tar-family formats (checked
    /// once per manifest entry inside `write_host_tar_archive`'s writer
    /// thread — a truncated tar is cleaned up via the same path as a write
    /// error) and, per-entry, inside `sync_host_dir_archive`'s dispatch/
    /// materialize pipeline for `Dir` — either way a cancel stops the host
    /// currently in progress without waiting for it to finish, and no
    /// further host starts.
    async fn archive_one_host(
        &self,
        task_id: &TaskId,
        state: &Data<Arc<ApiWorkerState>>,
        profile: &woodstock::config::ArchiveProfile,
        hostname: &str,
        backup: &woodstock::config::Backup,
        counters: &Arc<ArchiveProgressCounters>,
        cancel_token: CancellationToken,
    ) -> Result<Option<u64>> {
        match profile.format {
            woodstock::config::ArchiveFormat::Tar(_)
            | woodstock::config::ArchiveFormat::TarGz(_)
            | woodstock::config::ArchiveFormat::TarXz(_)
            | woodstock::config::ArchiveFormat::TarZstd(_) => {
                woodstock::archiving::tar_writer::write_host_tar_archive(
                    &state.backups,
                    &state.config.path.pool_path,
                    hostname,
                    backup,
                    &profile.destination,
                    profile.format,
                    Some(counters.clone()),
                    cancel_token,
                )
                .await
                .map(|output| {
                    info!(
                        "[{}] Archived {} -> {:?}",
                        task_id, hostname, output.archive_path
                    );
                    Some(output.archive_size)
                })
                .map_err(|e| eyre!(e.to_string()))
            }
            woodstock::config::ArchiveFormat::Dir => {
                woodstock::archiving::dir_sync::sync_host_dir_archive(
                    &state.backups,
                    &state.config.path.pool_path,
                    hostname,
                    backup,
                    &profile.destination,
                    Some(counters.clone()),
                    cancel_token,
                )
                .await
                .map_err(|e| eyre!(e.to_string()))
                .and_then(|output| {
                    info!(
                        "[{}] Synced {} -> {:?} (+{} ~{} -{}, skipped {}{})",
                        task_id,
                        hostname,
                        output.destination,
                        output.added,
                        output.modified,
                        output.removed,
                        output.skipped,
                        if output.cancelled { ", cancelled" } else { "" }
                    );
                    // Entries that failed to materialize (permission denied,
                    // missing pool chunk, ...) are logged as `error!` where
                    // they happen but don't abort the sync — every other
                    // entry still gets a best-effort attempt. Surfacing that
                    // here as an `Err` is what makes the host actually show
                    // up as failed (`ArchiveHostExecutionState::Failed` /
                    // `failed_hosts`, below) instead of a silent `Success`
                    // despite files never landing on disk.
                    if output.skipped > 0 {
                        Err(eyre!(
                            "{} entr{} failed to materialize (see archive log)",
                            output.skipped,
                            if output.skipped == 1 { "y" } else { "ies" }
                        ))
                    } else {
                        Ok(None)
                    }
                })
            }
        }
    }

    /// Spawns a ~1s ticker publishing `counters`' live bytes/files values for
    /// the host at `host_index` until aborted by the caller (when that
    /// host's write finishes). `template` is a snapshot of `run_state` taken
    /// right before this host started (already marked `InProgress` at
    /// `host_index`) — every tick re-derives just the live counts from it
    /// rather than tracking separate shared state for the rest of the run,
    /// which only changes at host boundaries that `handle_archive_run`
    /// publishes itself.
    #[allow(clippy::too_many_arguments)]
    fn spawn_archive_progress_ticker(
        &self,
        task_id: &TaskId,
        template: ArchiveState,
        host_index: usize,
        host_start_bytes: u64,
        host_start_files: usize,
        counters: &Arc<ArchiveProgressCounters>,
    ) -> tokio::task::JoinHandle<()> {
        let publisher = self.progress.clone();
        let task_id = task_id.clone();
        let counters = counters.clone();

        tokio::spawn(
            async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
                interval.tick().await; // first tick fires immediately — skip it
                loop {
                    interval.tick().await;
                    if let Some(publi) = &publisher {
                        let (total_bytes, total_files) = counters.snapshot();
                        let mut state = template.clone();
                        state.progress_current = total_bytes;
                        state.file_count = total_files;
                        state.host_states[host_index].progress_current =
                            total_bytes - host_start_bytes;
                        state.host_states[host_index].file_count = total_files - host_start_files;
                        if let Err(e) = publi
                            .update_progress(&task_id.to_string(), ProgressUpdate::Archive(state))
                            .await
                        {
                            error!("[{}] Failed to publish archive progress: {}", task_id, e);
                        }
                    }
                }
            }
            .in_current_span(),
        )
    }

    /// Publishes one `ArchiveState` snapshot immediately — used at host
    /// boundaries (start of run, right after each host finishes), where the
    /// ticker's up-to-1s polling delay isn't good enough.
    async fn publish_archive_progress(&self, task_id: &TaskId, state: &ArchiveState) {
        if let Some(publi) = &self.progress {
            if let Err(e) = publi
                .update_progress(&task_id.to_string(), ProgressUpdate::Archive(state.clone()))
                .await
            {
                error!("[{}] Failed to publish archive progress: {}", task_id, e);
            }
        }
    }

    /// Publishes a "cancelled" run state for an archive run that was
    /// cancelled while still queued — before any host was sized or started.
    /// Mirrors `publish_cancelled_backup`/`publish_cancelled_restore`'s role
    /// for the other two job kinds.
    async fn publish_cancelled_archive(&self, task_id: &TaskId, hostnames: &[String]) {
        if let Some(publi) = &self.progress {
            let cancelled_state = ArchiveState {
                cancelled: true,
                hosts_total: hostnames.len(),
                host_states: hostnames
                    .iter()
                    .map(|hostname| ArchiveHostState {
                        hostname: hostname.clone(),
                        execution_state: ArchiveHostExecutionState::Cancelled,
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            };
            if let Err(e) = publi
                .update_progress(
                    &task_id.to_string(),
                    ProgressUpdate::Archive(cancelled_state),
                )
                .await
            {
                error!(
                    "[{}] Failed to publish cancelled archive state: {}",
                    task_id, e
                );
            }
        }
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
