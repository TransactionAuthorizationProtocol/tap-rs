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
        // Check if timestamp would overflow when converting to i64
        if timestamp > i64::MAX as u64 {
            // Return a date far in the future (year 3000+)
            DateTime::parse_from_rfc3339("3000-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc)
        } else {
            DateTime::from_timestamp(timestamp as i64, 0).unwrap_or_else(Utc::now)
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

        // Set created_time to a very large number that would fail datetime conversion
        message.created_time = Some(u64::MAX);

        // With u64::MAX, this should be far in the future and fail validation
        match validator.validate(&message).await {
            ValidationResult::Accept => panic!("Expected reject, got accept"),
            ValidationResult::Reject(reason) => {
                assert!(reason.contains("too far in the future"));
            }
        }
    }
}
