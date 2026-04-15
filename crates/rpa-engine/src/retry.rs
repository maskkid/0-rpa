//! Retry logic with configurable backoff strategies.

use rpa_core::context::RetryConfig;
use rpa_core::error::Result;
use std::time::Duration;
use tokio::time::sleep;

/// Execute an async operation with retry logic.
///
/// Retries the operation up to `config.max_retries` times with the configured
/// backoff strategy. Returns the first successful result or the last error.
pub async fn retry<F, Fut, T>(config: &RetryConfig, operation: F) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut last_error = None;

    for attempt in 0..=config.max_retries {
        match operation().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                if attempt < config.max_retries {
                    let delay = config.delay_for_attempt(attempt);
                    tracing::debug!(
                        attempt = attempt + 1,
                        max_retries = config.max_retries,
                        delay_ms = delay,
                        error = %e,
                        "Retrying after failure"
                    );
                    sleep(Duration::from_millis(delay)).await;
                }
                last_error = Some(e);
            }
        }
    }

    Err(last_error.unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rpa_core::error::RpaError;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn retry_succeeds_first_try() {
        let config = RetryConfig {
            max_retries: 3,
            delay_ms: 10,
            backoff: rpa_core::context::BackoffStrategy::Fixed,
        };
        let result: Result<String> = retry(&config, || async { Ok("success".into()) }).await;
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn retry_succeeds_after_failures() {
        let config = RetryConfig {
            max_retries: 3,
            delay_ms: 1,
            backoff: rpa_core::context::BackoffStrategy::Fixed,
        };
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let result = retry(&config, || {
            let attempts = attempts_clone.clone();
            async move {
                let n = attempts.fetch_add(1, Ordering::SeqCst);
                if n < 2 {
                    Err(RpaError::ElementNotFound("not yet".into()))
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
    async fn retry_exhausts_retries() {
        let config = RetryConfig {
            max_retries: 2,
            delay_ms: 1,
            backoff: rpa_core::context::BackoffStrategy::Fixed,
        };

        let result: Result<String> = retry(&config, || async {
            Err(RpaError::ElementNotFound("always fail".into()))
        })
        .await;

        assert!(result.is_err());
    }
}