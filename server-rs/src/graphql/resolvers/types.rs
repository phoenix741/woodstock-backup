use async_graphql::{ComplexObject, Context, Object, ID};
use chrono::{DateTime, Local};

use super::super::scalars::BigIntScalar;
use crate::api::dto::{
    BackupShareRecord, BackupStatusDto, FileDescription, FileManifestTypeDto, Host,
    HostAvailibilityState, HostConfiguration, RetentionCategoryDto,
};
use crate::api::ApiServerState;
use crate::graphql::scalars::BufferScalar;

#[derive(Clone)]
pub struct BackupEx {
    pub hostname: String,
    pub inner: crate::api::dto::Backup,
    /// Retention category computed at query time. `None` for in-flight or
    /// single-backup lookups where the full list is not available.
    pub retention_category: Option<RetentionCategoryDto>,
}

#[Object]
impl BackupEx {
    async fn id(&self) -> ID {
        ID(self.inner.id.clone())
    }

    async fn number(&self) -> i32 {
        self.inner.number as i32
    }
    async fn status(&self) -> BackupStatusDto {
        self.inner.status.clone()
    }
    async fn error_count(&self) -> i32 {
        self.inner.error_count as i32
    }
    async fn error_message(&self) -> Option<String> {
        self.inner.error_message.clone()
    }
    async fn start_date(&self) -> DateTime<Local> {
        self.inner.start_date
    }
    async fn end_date(&self) -> Option<DateTime<Local>> {
        self.inner.end_date
    }
    async fn file_count(&self) -> i32 {
        self.inner.file_count as i32
    }
    async fn new_file_count(&self) -> i32 {
        self.inner.new_file_count as i32
    }
    async fn existing_file_count(&self) -> i32 {
        self.inner.existing_file_count as i32
    }
    async fn removed_file_count(&self) -> i32 {
        self.inner.removed_file_count as i32
    }
    async fn modified_file_count(&self) -> i32 {
        self.inner.modified_file_count as i32
    }
    async fn file_size(&self) -> BigIntScalar {
        BigIntScalar(self.inner.file_size)
    }
    async fn existing_file_size(&self) -> BigIntScalar {
        BigIntScalar(self.inner.existing_file_size)
    }
    async fn new_file_size(&self) -> BigIntScalar {
        BigIntScalar(self.inner.new_file_size)
    }
    async fn modified_file_size(&self) -> BigIntScalar {
        BigIntScalar(self.inner.modified_file_size)
    }
    async fn compressed_file_size(&self) -> BigIntScalar {
        BigIntScalar(self.inner.compressed_file_size)
    }
    async fn existing_compressed_file_size(&self) -> BigIntScalar {
        BigIntScalar(self.inner.existing_compressed_file_size)
    }
    async fn new_compressed_file_size(&self) -> BigIntScalar {
        BigIntScalar(self.inner.new_compressed_file_size)
    }
    async fn modified_compressed_file_size(&self) -> BigIntScalar {
        BigIntScalar(self.inner.modified_compressed_file_size)
    }
    async fn speed(&self) -> f64 {
        self.inner.speed
    }
    async fn agent_version(&self) -> Option<String> {
        self.inner.agent_version.clone()
    }
    async fn retention_category(&self) -> Option<RetentionCategoryDto> {
        self.retention_category
    }

    async fn shares(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<FileDescription>> {
        let state = ctx.data::<ApiServerState>()?;
        let mut files = (*state.files_service).clone();
        let backup_id = uuid::Uuid::parse_str(&self.inner.id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid backup ID: {e}")))?;
        let shares = files
            .list_shares(&self.hostname, backup_id)
            .await
            .map_err(super::util::map_err)?;

        Ok(shares.into_iter().map(Into::into).collect())
    }

    async fn share_records(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<BackupShareRecord>> {
        let state = ctx.data::<ApiServerState>()?;
        let backup_id = uuid::Uuid::parse_str(&self.inner.id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid backup ID: {e}")))?;
        let records = state
            .backups_service
            .get_backup_share_records(&self.hostname, backup_id)
            .await;
        Ok(records.into_iter().map(Into::into).collect())
    }

    async fn files(
        &self,
        ctx: &Context<'_>,
        share_path: String,
        path: BufferScalar,
    ) -> async_graphql::Result<Vec<FileDescription>> {
        let state = ctx.data::<ApiServerState>()?;
        let mut files = (*state.files_service).clone();

        let backup_id = uuid::Uuid::parse_str(&self.inner.id)
            .map_err(|e| async_graphql::Error::new(format!("Invalid backup ID: {e}")))?;
        let res = files
            .list_files(&self.hostname, backup_id, &share_path, &path.0)
            .await
            .map_err(super::util::map_err)?;
        Ok(res.into_iter().map(Into::into).collect())
    }
}

#[ComplexObject]
impl FileDescription {
    async fn r#type(&self) -> FileManifestTypeDto {
        self.stats
            .as_ref()
            .map(|f| f.r#type)
            .unwrap_or(FileManifestTypeDto::Unknown)
    }
}

#[ComplexObject]
impl Host {
    async fn configuration(&self, ctx: &Context<'_>) -> async_graphql::Result<HostConfiguration> {
        let state = ctx.data::<ApiServerState>()?;
        let config = state
            .hosts_service
            .get_public_host_configuration(&self.name)
            .await
            .map_err(super::util::map_err)?;
        Ok(config.into())
    }

    async fn backups(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<BackupEx>> {
        let state = ctx.data::<ApiServerState>()?;
        let raw_backups = state.backups.get_backups(&self.name).await;

        // Compute retention categories from the full backup list.
        let categories = compute_retention_categories(&self.name, state, &raw_backups).await;

        Ok(raw_backups
            .into_iter()
            .map(|b| {
                let id = b.id;
                let retention = categories.get(&id).copied().map(Into::into);
                BackupEx {
                    hostname: self.name.to_string(),
                    inner: b.into(),
                    retention_category: retention,
                }
            })
            .collect())
    }

    async fn last_backup(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<BackupEx>> {
        let state = ctx.data::<ApiServerState>()?;
        let last = state
            .backups_service
            .get_last_backup(&self.name)
            .await
            .map_err(super::util::map_err)?;

        Ok(last.map(|b| BackupEx {
            hostname: self.name.to_string(),
            inner: b,
            retention_category: None,
        }))
    }

    async fn time_since_last_backup(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<f64>> {
        let state = ctx.data::<ApiServerState>()?;
        let since = state
            .backups_service
            .get_time_since_last_backup(&self.name)
            .await;
        Ok(since.map(|d| d.num_seconds() as f64))
    }

    async fn time_to_next_backup(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<f64>> {
        let state = ctx.data::<ApiServerState>()?;
        let since = state
            .backups_service
            .get_time_to_next_backup(&self.name)
            .await
            .map_err(super::util::map_err)?;
        Ok(since.map(|d| d.num_seconds() as f64))
    }

    async fn date_to_next_backup(
        &self,
        ctx: &Context<'_>,
    ) -> Option<async_graphql::Result<DateTime<Local>>> {
        let fun = async || {
            let state = ctx.data::<ApiServerState>()?;
            state
                .backups_service
                .get_date_to_next_backup(&self.name)
                .await
                .map_err(super::util::map_err)
        };
        fun().await.transpose()
    }

    async fn agent_version(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<String>> {
        let state = ctx.data::<ApiServerState>()?;

        if let Ok(Some(info)) = state.resolver.get_informations(&self.name).await {
            if !info.version.is_empty() {
                return Ok(Some(info.version));
            }
        }
        let last = state
            .backups_service
            .get_last_backup(&self.name)
            .await
            .map_err(super::util::map_err)?;
        Ok(last.and_then(|b| b.agent_version))
    }

    async fn availibility_state(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Option<HostAvailibilityState>> {
        let state = ctx.data::<ApiServerState>()?;

        if let Ok(Some(info)) = state.resolver.get_informations(&self.name).await {
            let state = if info.is_online {
                HostAvailibilityState::Online
            } else {
                HostAvailibilityState::Offline
            };
            return Ok(Some(state));
        }

        let config = state
            .hosts_service
            .get_public_host_configuration(&self.name)
            .await
            .map_err(super::util::map_err)?;
        if config.addresses.is_some() {
            return Ok(Some(HostAvailibilityState::Unknown));
        }

        Ok(Some(HostAvailibilityState::Offline))
    }

    async fn addresses(&self, ctx: &Context<'_>) -> async_graphql::Result<Option<Vec<String>>> {
        let state = ctx.data::<ApiServerState>()?;
        let host_conf = state
            .hosts_service
            .get_private_host_configuration(&self.name)
            .await
            .map_err(super::util::map_err)?;

        match state
            .job_utility
            .resolve_from_config(&self.name, &host_conf)
            .await
        {
            Ok(addrs) => Ok(Some(
                addrs.into_iter().map(|sa| sa.ip().to_string()).collect(),
            )),
            Err(_) => Ok(None),
        }
    }
}

/// Compute a retention-category map for a host's backup list.
///
/// Returns an empty map if the host has no retention policy configured, which
/// means all backups will have `retention_category = None` in the API response
/// (no chip shown in the frontend).
pub(super) async fn compute_retention_categories(
    hostname: &str,
    state: &ApiServerState,
    raw_backups: &[woodstock::config::Backup],
) -> std::collections::HashMap<uuid::Uuid, woodstock::server::backup::retention::RetentionCategory>
{
    let policy = match state.hosts.get_schedule(hostname).await {
        Ok(s) => s.backup_to_keep,
        Err(_) => return std::collections::HashMap::new(),
    };

    match policy {
        Some(p) => woodstock::server::backup::retention::classify_backups(
            raw_backups,
            &p,
            chrono::Local::now(),
        ),
        None => std::collections::HashMap::new(),
    }
}
