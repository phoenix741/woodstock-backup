use async_graphql::{Context, Object, Result as GqlResult};
use uuid::Uuid;

use crate::api::dto::{ArchiveRunResponse, JobResponse, RestoreInput};
use crate::api::ApiServerState;
use crate::jobs::progress::{JobKind, JobStatus};
use crate::jobs::types::RestoreJobData;

#[derive(Default)]
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_backup(&self, ctx: &Context<'_>, hostname: String) -> GqlResult<JobResponse> {
        let state = ctx.data::<ApiServerState>()?;
        let hosts = state
            .hosts_service
            .list_hosts()
            .await
            .map_err(super::util::map_err)?;

        if !hosts.contains(&hostname) {
            return Err(async_graphql::Error::new(format!(
                "Can't find the host with the name {hostname}"
            )));
        }

        let mut producers = state.producers.lock().await;
        let id_opt = producers
            .enqueue_backup_unique(&hostname, true)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        let id = id_opt.unwrap_or_else(|| format!("backup::{}", hostname));

        Ok(JobResponse { id })
    }

    /// Triggers an archive profile run now — regardless of whether it is
    /// enabled or due (same semantics as `ws_console archive run`). If
    /// `host` is omitted, resolves the profile's full host selection and
    /// enqueues one job per selected host.
    #[graphql(name = "runArchive")]
    async fn run_archive(
        &self,
        ctx: &Context<'_>,
        profile: String,
        host: Option<String>,
    ) -> GqlResult<ArchiveRunResponse> {
        let state = ctx.data::<ApiServerState>()?;
        let mut producers = state.producers.lock().await;

        let job_ids = if let Some(host) = host {
            producers
                .enqueue_archive_run(&profile, vec![host])
                .await
                .map_err(|e| async_graphql::Error::new(e.to_string()))?
                .into_iter()
                .collect()
        } else {
            let archiving = woodstock::config::ArchivingConfig::new(state.config.clone());
            producers
                .enqueue_archive_profile(&archiving, &profile)
                .await
                .map_err(|e| async_graphql::Error::new(e.to_string()))?
        };

        Ok(ArchiveRunResponse { job_ids })
    }

    async fn remove_backup(
        &self,
        ctx: &Context<'_>,
        hostname: String,
        id: String,
    ) -> GqlResult<JobResponse> {
        let state = ctx.data::<ApiServerState>()?;

        let backup_id = Uuid::parse_str(&id)
            .map_err(|_| async_graphql::Error::new(format!("Invalid backup UUID: {id}")))?;

        let hosts = state
            .hosts_service
            .list_hosts()
            .await
            .map_err(super::util::map_err)?;

        if !hosts.contains(&hostname) {
            return Err(async_graphql::Error::new(format!(
                "Can't find the host with the name {hostname}"
            )));
        }

        let backup = state
            .backups_service
            .get_backup(&hostname, backup_id)
            .await
            .map_err(super::util::map_err)?;

        let backup = backup.ok_or_else(|| {
            async_graphql::Error::new(format!(
                "Can't find the backup {id} for the host {hostname}"
            ))
        })?;

        let mut producers = state.producers.lock().await;
        let job_id = producers
            .enqueue_remove(&hostname, backup_id, backup.number)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(JobResponse { id: job_id })
    }

    async fn restore_backup(
        &self,
        ctx: &Context<'_>,
        input: RestoreInput,
    ) -> GqlResult<JobResponse> {
        let state = ctx.data::<ApiServerState>()?;

        let hosts = state
            .hosts_service
            .list_hosts()
            .await
            .map_err(super::util::map_err)?;

        if !hosts.contains(&input.hostname) {
            return Err(async_graphql::Error::new(format!(
                "Can't find the host with the name {}",
                input.hostname
            )));
        }

        let backup_id = Uuid::parse_str(&input.id)
            .map_err(|_| async_graphql::Error::new(format!("Invalid backup UUID: {}", input.id)))?;

        let backup = state
            .backups_service
            .get_backup(&input.hostname, backup_id)
            .await
            .map_err(super::util::map_err)?;

        let backup = backup.ok_or_else(|| {
            async_graphql::Error::new(format!(
                "Can't find the backup {} for the host {}",
                input.id, input.hostname
            ))
        })?;

        let mut producers = state.producers.lock().await;

        let files = input
            .files
            .into_iter()
            .map(|f| crate::jobs::types::JobRestoreDataSelection {
                share: f.share,
                selection: f.selection,
            })
            .collect();

        let job = RestoreJobData {
            host: input.hostname,
            config: None,
            id: backup_id,
            number: backup.number,
            ip: None,
            start_date: None,
            destination_directory: input.destination_directory,
            files,
        };

        let id = producers
            .enqueue_restore(job)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(JobResponse { id })
    }

    /// Requests cancellation of a backup/restore/archive/fsck job, whether
    /// it's still waiting in the Apalis queue or already running. The job's
    /// owning `job_worker` process (possibly a different process than this
    /// one) picks up the request via a Redis marker it polls — see
    /// `jobs::progress::request_cancel`/`spawn_cancel_watcher`. Returns
    /// `false` (no-op) if the job is unknown or has already reached a
    /// terminal status.
    #[graphql(name = "cancelJob")]
    async fn cancel_job(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "taskId")] task_id: String,
    ) -> GqlResult<bool> {
        let state = ctx.data::<ApiServerState>()?;

        let Some(event) = state.progress_reader.get(&task_id).await else {
            return Ok(false);
        };
        // Only Backup/Restore/Archive/Fsck handlers poll the cancel marker —
        // a Remove/Stats/CleanupRefcnt job would run to completion while
        // this mutation lied about having stopped it.
        if !matches!(
            event.kind,
            JobKind::Backup(_) | JobKind::Restore(_) | JobKind::Archive(_) | JobKind::Fsck(_)
        ) {
            return Ok(false);
        }
        if !matches!(event.status, JobStatus::Created | JobStatus::Started) {
            return Ok(false);
        }

        crate::jobs::progress::request_cancel(&state.redis_client, &task_id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(true)
    }

    #[graphql(name = "cleanupPool")]
    async fn cleanup_pool(&self, ctx: &Context<'_>) -> GqlResult<JobResponse> {
        let state = ctx.data::<ApiServerState>()?;
        let mut producers = state.producers.lock().await;
        let id = producers
            .enqueue_cleanup_refcnt()
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(JobResponse { id })
    }

    #[graphql(name = "checkAndFixPool")]
    async fn check_and_fix_pool(
        &self,
        ctx: &Context<'_>,
        #[graphql(name = "fix")] fix: bool,
        #[graphql(name = "verifyChunks")] verify_chunks: bool,
    ) -> GqlResult<JobResponse> {
        let state = ctx.data::<ApiServerState>()?;
        let mut producers = state.producers.lock().await;
        let dry_run = !fix;
        let id = producers
            .enqueue_fsck(dry_run, verify_chunks)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(JobResponse { id })
    }

    #[graphql(name = "clearCache")]
    async fn clear_cache(&self, ctx: &Context<'_>) -> GqlResult<JobResponse> {
        let state = ctx.data::<ApiServerState>()?;
        state
            .server_service
            .clear_cache()
            .await
            .map_err(super::util::map_err)?;
        Ok(JobResponse { id: "ok".into() })
    }

    /// Déclenche manuellement la purge des sauvegardes en surplus pour un hôte.
    ///
    /// Calcule immédiatement les sauvegardes à supprimer selon la politique de rétention
    /// configurée pour l'hôte, puis enqueue un job `Remove` pour chacune.
    #[graphql(name = "purgeRetention")]
    async fn purge_retention(&self, ctx: &Context<'_>, hostname: String) -> GqlResult<JobResponse> {
        let state = ctx.data::<ApiServerState>()?;

        let hosts = state
            .hosts_service
            .list_hosts()
            .await
            .map_err(super::util::map_err)?;

        if !hosts.contains(&hostname) {
            return Err(async_graphql::Error::new(format!(
                "Can't find the host with the name {hostname}"
            )));
        }

        let mut producers = state.producers.lock().await;
        producers
            .enforce_retention_for_host(&hostname, None)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(JobResponse {
            id: format!("purge::{hostname}"),
        })
    }
}
