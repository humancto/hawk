use std::future::Future;
use std::time::Duration;
use tracing::warn;

const MAX_RETRIES: u32 = 3;
const BASE_DELAY_MS: u64 = 500;

/// Retry an async operation with exponential backoff.
/// Retries on any error up to MAX_RETRIES times.
pub async fn with_retry<F, Fut, T, E>(op_name: &str, mut f: F) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    loop {
        match f().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                attempt += 1;
                if attempt > MAX_RETRIES {
                    return Err(e);
                }
                let delay = Duration::from_millis(BASE_DELAY_MS * 2u64.pow(attempt - 1));
                warn!(
                    "{op_name} failed (attempt {attempt}/{MAX_RETRIES}): {e}. Retrying in {}ms...",
                    delay.as_millis()
                );
                tokio::time::sleep(delay).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_retry_succeeds_on_third_attempt() {
        let attempts = AtomicU32::new(0);
        let result: Result<&str, String> = with_retry("test_op", || {
            let count = attempts.fetch_add(1, Ordering::SeqCst);
            async move {
                if count < 2 {
                    Err(format!("fail #{count}"))
                } else {
                    Ok("success")
                }
            }
        })
        .await;

        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_exhausts_and_returns_last_error() {
        let attempts = AtomicU32::new(0);
        let result: Result<(), String> = with_retry("test_op", || {
            attempts.fetch_add(1, Ordering::SeqCst);
            async { Err("always fails".to_string()) }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 4); // 1 initial + 3 retries
    }

    #[tokio::test]
    async fn test_retry_immediate_success() {
        let result: Result<i32, String> =
            with_retry("test_op", || async { Ok(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }
}
