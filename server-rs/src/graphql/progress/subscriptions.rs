use async_graphql::{Context, Subscription};
use futures::{Stream, StreamExt};

use crate::{
    api::{dto::Job, state::ApiServerState},
    graphql::resolvers::types::BackupEx,
    jobs::progress::ProgressFilter,
};
use woodstock::config::{BackupChangedEvent, BACKUP_CHANGED_CHANNEL};

/// Returns `true` when a [`BackupChangedEvent`] should be forwarded to the subscriber.
///
/// Two conditions must hold simultaneously:
/// - The event belongs to the subscribed `hostname` (events are broadcast to all subscribers).
/// - The backup was **not** removed. Removals are signalled here but the backup no longer exists
///   on disk, so fetching it would fail. The front-end handles that case separately via the
///   `jobUpdated(kind: "remove")` subscription.
fn should_emit(event: &BackupChangedEvent, hostname: &str) -> bool {
    event.hostname == hostname && !event.removed
}

#[derive(Default)]
pub struct ProgressSubscription;

#[Subscription]
impl ProgressSubscription {
    /// Subscription: job updates with optional `host` and `kind` filters.
    /// Without filters, all jobs are returned (historical behaviour).
    #[graphql(name = "jobUpdated")]
    async fn job_updated(
        &self,
        ctx: &Context<'_>,
        host: Option<String>,
        kind: Option<String>,
    ) -> impl Stream<Item = Job> {
        let state = ctx.data_unchecked::<ApiServerState>().clone();
        let filter = ProgressFilter {
            host,
            kind,
            ..Default::default()
        };
        let stream = state.progress_reader.stream(filter).await;
        stream.map(|event| Job::from(event))
    }

    /// Subscription: real backup changes for a given host.
    ///
    /// Emitted via Redis Pub/Sub (`woodstock:backup:changed`) whenever
    /// `backup.yml` is written to disk by any server process
    /// (job_worker, api_server, …):
    /// - Creation or update (`add_or_replace_backup`, `update_backup`)
    /// - Removal signalled (`remove_backup`)
    ///
    /// For completed removals (backup no longer on disk), the front-end
    /// uses the `jobUpdated(kind: "remove")` subscription and triggers a refetch.
    #[graphql(name = "backupUpdated")]
    async fn backup_updated(
        &self,
        ctx: &Context<'_>,
        hostname: String,
    ) -> impl Stream<Item = BackupEx> {
        use futures::stream;
        use tracing::error;

        let state = ctx.data_unchecked::<ApiServerState>().clone();

        let mut pubsub = match state.redis_client.get_async_pubsub().await {
            Ok(p) => p,
            Err(e) => {
                error!("backupUpdated: failed to get Redis pubsub: {}", e);
                return stream::empty().boxed();
            }
        };

        if let Err(e) = pubsub.subscribe(BACKUP_CHANGED_CHANNEL).await {
            error!(
                "backupUpdated: failed to subscribe to {}: {}",
                BACKUP_CHANGED_CHANNEL, e
            );
            return stream::empty().boxed();
        }

        pubsub
            .into_on_message()
            .filter_map(move |msg| {
                let hostname = hostname.clone();
                let state = state.clone();
                async move {
                    let payload = msg.get_payload::<String>().ok()?;
                    let event: BackupChangedEvent = serde_json::from_str(&payload).ok()?;
                    if !should_emit(&event, &hostname) {
                        return None;
                    }
                    state
                        .backups_service
                        .get_backup(&hostname, event.backup_id)
                        .await
                        .ok()
                        .flatten()
                        .map(|backup| BackupEx {
                            hostname,
                            inner: backup,
                        })
                }
            })
            .boxed()
    }
}
