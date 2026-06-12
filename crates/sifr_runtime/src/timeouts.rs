use std::time::Duration;

const MAX_RUNTIME_TIMEOUT_SECONDS: f64 = 86_400.0;

pub(crate) fn timeout_duration(seconds: f64, label: &str) -> Result<Duration, String> {
    if !seconds.is_finite() || seconds <= 0.0 {
        return Err(format!("{label} timeout must be finite and positive"));
    }
    if seconds > MAX_RUNTIME_TIMEOUT_SECONDS {
        return Err(format!("{label} timeout is too large"));
    }
    Ok(Duration::from_secs_f64(seconds))
}

#[cfg(test)]
mod tests {
    use super::timeout_duration;

    #[test]
    fn rejects_non_finite_non_positive_and_overflow_sized_timeouts() {
        assert_eq!(
            timeout_duration(f64::NAN, "network").err().as_deref(),
            Some("network timeout must be finite and positive")
        );
        assert_eq!(
            timeout_duration(0.0, "network").err().as_deref(),
            Some("network timeout must be finite and positive")
        );
        assert_eq!(
            timeout_duration(1e20, "network").err().as_deref(),
            Some("network timeout is too large")
        );
    }

    #[test]
    fn accepts_bounded_finite_positive_timeout() {
        assert!(timeout_duration(2.0, "network").is_ok());
        assert!(timeout_duration(86_400.0, "HTTP").is_ok());
    }
}
