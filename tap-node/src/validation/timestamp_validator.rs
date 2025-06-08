//! Timestamp validation for TAP messages

use super::{MessageValidator, ValidationResult};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use tap_msg::didcomm::PlainMessage;

/// Validator that checks message timestamps
///
/// This validator ensures that:
/// - Messages are not too far in the future (prevents clock drift issues)
/// - Messages have not expired
/// - Timestamps are valid and parseable
pub struct TimestampValidator {
    max_future_drift_secs: i64,
}

impl TimestampValidator {
    /// Create a new timestamp validator
    ///
    /// # Arguments
    /// * `max_future_drift_secs` - Maximum allowed seconds a message can be from the future
    pub fn new(max_future_drift_secs: i64) -> Self {
        Self {
            max_future_drift_secs,
        }
    }

    /// Convert a Unix timestamp to DateTime
    fn timestamp_to_datetime(timestamp: u64) -> DateTime<Utc> {
        // Detect if timestamp is in seconds or milliseconds
        // Timestamps in seconds since 1970 are much smaller than timestamps in milliseconds
        // A reasonable cutoff is 10^10 (around year 2286 in seconds, or year 1970 + 4 months in milliseconds)
        let timestamp_seconds = if timestamp < 10_000_000_000 {
            // Timestamp is likely in seconds
            timestamp
        } else {
            // Timestamp is likely in milliseconds, convert to seconds
            timestamp / 1000
        };

        // Check if timestamp would overflow when converting to i64
        if timestamp_seconds > i64::MAX as u64 {
            // Return a date far in the future (year 3000+)
            DateTime::parse_from_rfc3339("3000-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc)
        } else {
            DateTime::from_timestamp(timestamp_seconds as i64, 0).unwrap_or_else(Utc::now)
        }
    }
}

#[async_trait]
impl MessageValidator for TimestampValidator {
    async fn validate(&self, message: &PlainMessage) -> ValidationResult {
        let now = Utc::now();

        // Check created_time
        if let Some(created_time) = message.created_time {
            let created_dt = Self::timestamp_to_datetime(created_time);

            // Check if message is too far in the future
            let max_future = now + Duration::seconds(self.max_future_drift_secs);
            if created_dt > max_future {
                return ValidationResult::Reject(format!(
                    "Message created_time is too far in the future: {} (max allowed: {})",
                    created_dt, max_future
                ));
            }
        }

        // Check expires_time if present
        if let Some(expires_time) = message.expires_time {
            let expires_dt = Self::timestamp_to_datetime(expires_time);

            // Check if message has expired
            if now > expires_dt {
                return ValidationResult::Reject(format!(
                    "Message has expired at: {} (current time: {})",
                    expires_dt, now
                ));
            }
        }

        ValidationResult::Accept
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[tokio::test]
    async fn test_valid_timestamp() {
        let validator = TimestampValidator::new(60);
        let message = PlainMessage::new(
            "test_msg_1".to_string(),
            "test_type".to_string(),
            serde_json::json!({}),
            "did:example:sender".to_string(),
        )
        .with_recipient("did:example:receiver");

        // created_time is set automatically in PlainMessage::new, which uses current time

        match validator.validate(&message).await {
            ValidationResult::Accept => {} // Expected
            ValidationResult::Reject(reason) => panic!("Expected accept, got reject: {}", reason),
        }
    }

    #[tokio::test]
    async fn test_future_timestamp_within_drift() {
        let validator = TimestampValidator::new(60);
        let mut message = PlainMessage::new(
            "test_msg_2".to_string(),
            "test_type".to_string(),
            serde_json::json!({}),
            "did:example:sender".to_string(),
        )
        .with_recipient("did:example:receiver");

        // Set created_time to 30 seconds in the future (within allowed drift)
        let future_time = Utc::now() + Duration::seconds(30);
        message.created_time = Some(future_time.timestamp() as u64);

        match validator.validate(&message).await {
            ValidationResult::Accept => {} // Expected
            ValidationResult::Reject(reason) => panic!("Expected accept, got reject: {}", reason),
        }
    }

    #[tokio::test]
    async fn test_future_timestamp_exceeds_drift() {
        let validator = TimestampValidator::new(60);
        let mut message = PlainMessage::new(
            "test_msg_2".to_string(),
            "test_type".to_string(),
            serde_json::json!({}),
            "did:example:sender".to_string(),
        )
        .with_recipient("did:example:receiver");

        // Set created_time to 2 minutes in the future (exceeds allowed drift)
        let future_time = Utc::now() + Duration::seconds(120);
        message.created_time = Some(future_time.timestamp() as u64);

        match validator.validate(&message).await {
            ValidationResult::Accept => panic!("Expected reject, got accept"),
            ValidationResult::Reject(reason) => {
                assert!(reason.contains("too far in the future"));
            }
        }
    }

    #[tokio::test]
    async fn test_expired_message() {
        let validator = TimestampValidator::new(60);
        let mut message = PlainMessage::new(
            "test_msg_2".to_string(),
            "test_type".to_string(),
            serde_json::json!({}),
            "did:example:sender".to_string(),
        )
        .with_recipient("did:example:receiver");

        // Set expires_time to 1 minute ago
        let expired_time = Utc::now() - Duration::seconds(60);
        message.expires_time = Some(expired_time.timestamp() as u64);

        match validator.validate(&message).await {
            ValidationResult::Accept => panic!("Expected reject, got accept"),
            ValidationResult::Reject(reason) => {
                assert!(reason.contains("has expired"));
            }
        }
    }

    #[tokio::test]
    async fn test_very_large_timestamp() {
        let validator = TimestampValidator::new(60);
        let mut message = PlainMessage::new(
            "test_msg_2".to_string(),
            "test_type".to_string(),
            serde_json::json!({}),
            "did:example:sender".to_string(),
        )
        .with_recipient("did:example:receiver");

        // Set created_time to year 3000 in milliseconds (well beyond reasonable future)
        // 3000-01-01 = approximately 32503680000 seconds = 32503680000000 milliseconds
        message.created_time = Some(32503680000000);

        // This should be far in the future and fail validation
        match validator.validate(&message).await {
            ValidationResult::Accept => panic!("Expected reject, got accept"),
            ValidationResult::Reject(reason) => {
                assert!(reason.contains("too far in the future"));
            }
        }
    }

    #[tokio::test]
    async fn test_timestamp_milliseconds() {
        let validator = TimestampValidator::new(60);
        let mut message = PlainMessage::new(
            "test_msg_3".to_string(),
            "test_type".to_string(),
            serde_json::json!({}),
            "did:example:sender".to_string(),
        )
        .with_recipient("did:example:receiver");

        // Set created_time to current time in milliseconds
        message.created_time = Some(Utc::now().timestamp_millis() as u64);

        match validator.validate(&message).await {
            ValidationResult::Accept => {} // Expected
            ValidationResult::Reject(reason) => panic!("Expected accept, got reject: {}", reason),
        }
    }

    #[tokio::test]
    async fn test_future_timestamp_milliseconds_exceeds_drift() {
        let validator = TimestampValidator::new(60);
        let mut message = PlainMessage::new(
            "test_msg_4".to_string(),
            "test_type".to_string(),
            serde_json::json!({}),
            "did:example:sender".to_string(),
        )
        .with_recipient("did:example:receiver");

        // Set created_time to 2 minutes in the future in milliseconds (exceeds allowed drift)
        let future_time = Utc::now() + Duration::seconds(120);
        message.created_time = Some(future_time.timestamp_millis() as u64);

        match validator.validate(&message).await {
            ValidationResult::Accept => panic!("Expected reject, got accept"),
            ValidationResult::Reject(reason) => {
                assert!(reason.contains("too far in the future"));
            }
        }
    }
}
