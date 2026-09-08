use std::future::Future;
use std::time::Duration;

use tracing::warn;

/// Retries a transient Redis operation up to 2 more times (50 ms then 200 ms delay) before
/// giving up. A long-held `ConnectionManager` can hand back a stale command after a network
/// hiccup (it reconnects in the background, but the in-flight command on the old socket
/// still fails); a freshly opened connection can also hit a one-off network glitch.
/// Centralizes the retry policy so it isn't duplicated at every call site.
pub async fn retry_transient<T, F, Fut>(context: &str, mut op: F) -> redis::RedisResult<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = redis::RedisResult<T>>,
{
    const RETRY_DELAYS_MS: [u64; 2] = [50, 200];
    let mut attempt = 0;
    loop {
        match op().await {
            Ok(v) => return Ok(v),
            Err(err) if attempt < RETRY_DELAYS_MS.len() => {
                warn!(
                    "{context}: redis operation failed (attempt {}/{}), retrying: {err}",
                    attempt + 1,
                    RETRY_DELAYS_MS.len() + 1,
                );
                tokio::time::sleep(Duration::from_millis(RETRY_DELAYS_MS[attempt])).await;
                attempt += 1;
            }
            Err(err) => return Err(err),
        }
    }
}
