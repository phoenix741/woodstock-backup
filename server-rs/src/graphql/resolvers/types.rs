use async_graphql::{ComplexObject, Context, Object, ID};
use chrono::{DateTime, Local};

use super::super::scalars::BigIntScalar;
use crate::api::dto::{
    BackupStatusDto, FileDescription, FileManifestTypeDto, Host, HostAvailibilityState,
    HostConfiguration,
};
use crate::api::ApiServerState;
use crate::graphql::scalars::BufferScalar;

#[derive(Clone)]
pub struct BackupEx {
    pub hostname: String,
    pub inner: crate::api::dto::Backup,
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
        let backups = state
            .backups_service
            .get_backups(&self.name)
            .await
            .map_err(super::util::map_err)?;

        Ok(backups
            .into_iter()
            .map(|b| BackupEx {
                hostname: self.name.to_string(),
                inner: b,
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
