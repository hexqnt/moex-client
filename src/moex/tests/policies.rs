use std::num::NonZeroU32;
use std::time::{Duration, Instant};

use reqwest::StatusCode;

use crate::moex::*;

#[test]
fn moex_error_helpers_detect_retryable_status_and_body_prefix() {
    let err = MoexError::HttpStatus {
        endpoint: "engines.json".into(),
        status: StatusCode::TOO_MANY_REQUESTS,
        content_type: Some("application/json".into()),
        body_prefix: "{\"error\":\"rate limit\"}".into(),
    };

    assert!(err.is_retryable());
    assert_eq!(err.status_code(), Some(StatusCode::TOO_MANY_REQUESTS));
    assert_eq!(
        err.response_body_prefix(),
        Some("{\"error\":\"rate limit\"}")
    );
}

#[test]
fn moex_error_helpers_mark_non_retryable_client_status() {
    let err = MoexError::HttpStatus {
        endpoint: "engines.json".into(),
        status: StatusCode::BAD_REQUEST,
        content_type: Some("application/json".into()),
        body_prefix: "{\"error\":\"bad request\"}".into(),
    };

    assert!(!err.is_retryable());
    assert_eq!(err.status_code(), Some(StatusCode::BAD_REQUEST));
}

#[test]
fn moex_error_helpers_return_body_prefix_for_non_json_payload() {
    let err = MoexError::NonJsonPayload {
        endpoint: "engines.json".into(),
        content_type: Some("text/html".into()),
        body_prefix: "<!doctype html>".into(),
    };

    assert_eq!(err.response_body_prefix(), Some("<!doctype html>"));
    assert_eq!(err.status_code(), None);
    assert!(!err.is_retryable());
}

#[test]
fn with_retry_retries_retryable_errors_until_success() {
    let policy =
        RetryPolicy::new(NonZeroU32::new(3).expect("non-zero attempts")).with_delay(Duration::ZERO);
    let mut attempts = 0;

    let value = with_retry(policy, || {
        attempts += 1;
        if attempts < 3 {
            return Err(MoexError::HttpStatus {
                endpoint: "engines.json".into(),
                status: StatusCode::TOO_MANY_REQUESTS,
                content_type: Some("application/json".into()),
                body_prefix: "{\"error\":\"rate limit\"}".into(),
            });
        }
        Ok(42_u32)
    })
    .expect("third attempt must succeed");

    assert_eq!(value, 42_u32);
    assert_eq!(attempts, 3);
}

#[test]
fn retry_policy_default_matches_example_values() {
    let policy = RetryPolicy::default();
    assert_eq!(policy.max_attempts().get(), 3);
    assert_eq!(policy.delay(), Duration::from_millis(400));
}

#[test]
fn with_retry_stops_on_non_retryable_error_without_extra_attempts() {
    let policy =
        RetryPolicy::new(NonZeroU32::new(5).expect("non-zero attempts")).with_delay(Duration::ZERO);
    let mut attempts = 0;

    let err = with_retry::<(), _>(policy, || {
        attempts += 1;
        Err(MoexError::HttpStatus {
            endpoint: "engines.json".into(),
            status: StatusCode::BAD_REQUEST,
            content_type: Some("application/json".into()),
            body_prefix: "{\"error\":\"bad request\"}".into(),
        })
    })
    .expect_err("non-retryable error must be returned immediately");

    assert!(matches!(
        err,
        MoexError::HttpStatus {
            status: StatusCode::BAD_REQUEST,
            ..
        }
    ));
    assert_eq!(attempts, 1);
}

#[test]
fn with_retry_stops_after_max_attempts_on_retryable_error() {
    let policy =
        RetryPolicy::new(NonZeroU32::new(2).expect("non-zero attempts")).with_delay(Duration::ZERO);
    let mut attempts = 0;

    let err = with_retry::<(), _>(policy, || {
        attempts += 1;
        Err(MoexError::HttpStatus {
            endpoint: "engines.json".into(),
            status: StatusCode::SERVICE_UNAVAILABLE,
            content_type: Some("application/json".into()),
            body_prefix: "{\"error\":\"temporary unavailable\"}".into(),
        })
    })
    .expect_err("retryable error must stop after max attempts");

    assert!(matches!(
        err,
        MoexError::HttpStatus {
            status: StatusCode::SERVICE_UNAVAILABLE,
            ..
        }
    ));
    assert_eq!(attempts, 2);
}

#[test]
fn rate_limit_per_second_rounds_up_interval() {
    let limit = RateLimit::per_second(NonZeroU32::new(3).expect("non-zero requests per second"));
    assert_eq!(limit.min_interval(), Duration::from_nanos(333_333_334));
}

#[test]
fn rate_limiter_reserve_delay_at_tracks_slots() {
    let mut limiter = RateLimiter::new(RateLimit::every(Duration::from_millis(100)));
    let start = Instant::now();

    assert_eq!(limiter.reserve_delay_at(start), Duration::ZERO);
    assert_eq!(limiter.reserve_delay_at(start), Duration::from_millis(100));
    assert_eq!(
        limiter.reserve_delay_at(start + Duration::from_millis(250)),
        Duration::ZERO
    );
    assert_eq!(
        limiter.reserve_delay_at(start + Duration::from_millis(300)),
        Duration::from_millis(50)
    );
}

#[test]
fn with_rate_limit_executes_action_and_returns_value() {
    let mut limiter = RateLimiter::new(RateLimit::every(Duration::ZERO));
    let value = with_rate_limit(&mut limiter, || 42_u32);
    assert_eq!(value, 42_u32);
}

#[cfg(feature = "async")]
fn block_on_immediate<F>(future: F) -> F::Output
where
    F: std::future::Future,
{
    use std::pin::pin;
    use std::task::{Context, Poll, Waker};

    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    let mut future = pin!(future);
    match future.as_mut().poll(&mut cx) {
        Poll::Ready(output) => output,
        Poll::Pending => panic!("test future unexpectedly returned Poll::Pending"),
    }
}

#[cfg(feature = "async")]
#[test]
fn with_rate_limit_async_skips_sleep_on_first_slot() {
    let mut limiter = RateLimiter::new(RateLimit::every(Duration::from_secs(1)));
    let mut sleep_calls = 0_u32;

    let value = block_on_immediate(with_rate_limit_async(
        &mut limiter,
        || std::future::ready(7_u32),
        |_delay| {
            sleep_calls += 1;
            std::future::ready(())
        },
    ));

    assert_eq!(value, 7_u32);
    assert_eq!(sleep_calls, 0);
}

#[cfg(feature = "async")]
#[test]
fn with_rate_limit_async_uses_injected_sleep_when_delay_is_reserved() {
    let mut limiter = RateLimiter::new(RateLimit::every(Duration::from_secs(1)));
    limiter.reserve_delay_at(Instant::now());
    let mut slept = Vec::new();

    let value = block_on_immediate(with_rate_limit_async(
        &mut limiter,
        || std::future::ready(9_u32),
        |delay| {
            slept.push(delay);
            std::future::ready(())
        },
    ));

    assert_eq!(value, 9_u32);
    assert_eq!(slept.len(), 1);
    assert!(slept[0] > Duration::ZERO);
    assert!(slept[0] <= Duration::from_secs(1));
}
