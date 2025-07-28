//! Middleware Tower pour publier Started/Completed/Failed automatiquement

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use apalis::prelude::Request;
use tower::{Layer, Service};
use tracing::debug;

use crate::jobs::progress::{JobStatus, ProgressPublisher};

/// Layer qui enveloppe un service Apalis et publie les statuts Started/Completed/Failed
#[derive(Clone)]
pub struct ProgressLayer {
    publisher: ProgressPublisher,
}

impl ProgressLayer {
    pub fn new(publisher: ProgressPublisher) -> Self {
        Self { publisher }
    }
}

impl<S> Layer<S> for ProgressLayer {
    type Service = ProgressService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ProgressService {
            inner,
            publisher: self.publisher.clone(),
        }
    }
}

/// Service Tower enveloppant l'appel du handler
#[derive(Clone)]
pub struct ProgressService<S> {
    inner: S,
    publisher: ProgressPublisher,
}

impl<S, J, C, E> Service<Request<J, C>> for ProgressService<S>
where
    S: Service<Request<J, C>, Error = E> + 'static,
    S::Future: Send + 'static,
    S::Response: Send + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    type Response = S::Response;
    type Error = E;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<J, C>) -> Self::Future
    where
        <S as Service<Request<J, C>>>::Response: Send,
    {
        // Récupère l'identifiant du job depuis les parties de la requête
        let task_id_str = req.parts.task_id.to_string();

        // Prépare l'appel interne (ne démarre pas tant qu'on n'attend pas le futur)
        let fut = self.inner.call(req);
        let publisher = self.publisher.clone();
        Box::pin(run_with_status::<_, _, _>(publisher, task_id_str, fut))
    }
}

// Fonction principale extraite du Box::pin: garde le même contrat de sortie
async fn run_with_status<R, E, F>(
    publisher: ProgressPublisher,
    task_id_str: String,
    fut: F,
) -> Result<R, E>
where
    F: Future<Output = Result<R, E>> + Send + 'static,
    R: Send + 'static,

    E: std::error::Error + Send + Sync + 'static,
{
    // Publie Started avant d'exécuter le handler, log en cas d'échec
    debug!("Marking job {} as Started", task_id_str);
    if let Err(e) = publisher
        .update_status(&task_id_str, JobStatus::Started)
        .await
    {
        debug!("Failed to publish Started for job {}: {}", task_id_str, e);
    }

    debug!("Running job handler for job {}", task_id_str);
    let result = fut.await;

    match &result {
        Ok(_) => {
            // debug!("Publish last event for job {}", task_id_str);
            // publisher
            //     .update_progress(&task_id_str, ProgressUpdate::Backup(state.clone()))
            //     .await?;
            debug!("Marking job {} as Completed", task_id_str);
            if let Err(e) = publisher.mark_completed(&task_id_str).await {
                debug!("Failed to publish Completed for job {}: {}", task_id_str, e);
            }
        }
        Err(err) => {
            debug!("Marking job {} as Failed with reason {}", task_id_str, err);
            let msg = err.to_string();
            if let Err(e) = publisher.mark_failed(&task_id_str, &msg).await {
                debug!(
                    "Failed to publish Failed for job {} ({}): {}",
                    task_id_str, msg, e
                );
            }
        }
    }

    result
}
