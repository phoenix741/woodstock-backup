use std::future::Future;
use std::time::Duration;

use tracing::warn;

/// Retries a transient Redis operation up to 2 more times (50 ms then 200 ms delay) before
/// giving up. A long-held `ConnectionManager` can hand back a stale command after a network
/// hiccup (it reconnects in the background, but the in-flight command on the old socket
/// still fails); a freshly opened connection can also hit a one-off network glitch.
/// Centralizes the retry policy so it isn't duplicated at every call site.
pub async fn retry_transient<T, F, Fut>(context: &str, op: F) -> redis::RedisResult<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = redis::RedisResult<T>>,
{
    retry_transient_with_attempts(context, op)
        .await
        .map(|(v, _retries)| v)
}

/// Same as [`retry_transient`], but also returns how many retries were actually performed
/// (0 = succeeded on the first attempt). Callers that need to tell genuine contention apart
/// from a possible lost server-side acknowledgment (e.g. a `SET NX` that actually landed,
/// but whose reply never reached this client) should use this instead — a `None`/no-op
/// result reached after at least one retry is ambiguous in a way a first-try result is not.
pub async fn retry_transient_with_attempts<T, F, Fut>(
    context: &str,
    mut op: F,
) -> redis::RedisResult<(T, u32)>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = redis::RedisResult<T>>,
{
    const RETRY_DELAYS_MS: [u64; 2] = [50, 200];
    let mut attempt = 0;
    loop {
        match op().await {
            Ok(v) => return Ok((v, attempt)),
            Err(err) if (attempt as usize) < RETRY_DELAYS_MS.len() => {
                warn!(
                    "{context}: redis operation failed (attempt {}/{}), retrying: {err}",
                    attempt + 1,
                    RETRY_DELAYS_MS.len() + 1,
                );
                tokio::time::sleep(Duration::from_millis(RETRY_DELAYS_MS[attempt as usize])).await;
                attempt += 1;
            }
            Err(err) => return Err(err),
        }
    }
}
