use futures::Future;
use tokio::time::{sleep, Duration};
use tracing::warn;

pub async fn retry<T, E, Fut>(f: impl Fn() -> Fut) -> T
where
    E: std::fmt::Display,
    Fut: Future<Output = Result<T, E>>,
{
    retry_times(f, Some, None).await.unwrap()
}

pub async fn retry_if<T, E, U, Fut>(f: impl Fn() -> Fut, map_done: impl Fn(T) -> Option<U>) -> U
where
    E: std::fmt::Display,
    Fut: Future<Output = Result<T, E>>,
{
    retry_times(f, map_done, None).await.unwrap()
}

pub async fn retry_times<T, E, U, Fut>(
    f: impl Fn() -> Fut,
    map_done: impl Fn(T) -> Option<U>,
    limit: Option<u64>,
) -> Result<U, RetriesExceeded>
where
    E: std::fmt::Display,
    Fut: Future<Output = Result<T, E>>,
{
    let mut failures = 0;
    loop {
        match limit {
            Some(limit) if failures >= limit => return Err(RetriesExceeded),
            _ => {}
        }
        match f().await.map(&map_done) {
            Ok(Some(val)) => return Ok(val),
            Err(e) => warn!("failed: {e}"),
            _ => {}
        }
        failures += 1;
        sleep(Duration::from_millis(1500)).await;
    }
}

#[derive(Clone, Copy, Debug, Default, thiserror::Error)]
#[error("retries exceeded")]
pub struct RetriesExceeded;
