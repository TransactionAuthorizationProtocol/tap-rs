//! Tests for Trust Ping processor integration

#[cfg(test)]
mod tests {
    use crate::event::{EventBus, EventSubscriber, NodeEvent};
    use crate::message::processor::PlainMessageProcessor;
    use crate::message::trust_ping_processor::TrustPingProcessor;
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use tap_msg::didcomm::PlainMessage;
    use tap_msg::message::tap_message_trait::TapMessageBody;
    use tap_msg::message::{TrustPing, TrustPingResponse};
    use tokio::time::{sleep, Duration};

    /// Test event subscriber that captures Trust Ping response events
    #[derive(Debug)]
    struct TestTrustPingSubscriber {
        received_responses: Arc<Mutex<Vec<PlainMessage>>>,
    }

    impl TestTrustPingSubscriber {
        fn new() -> Self {
            Self {
                received_responses: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn get_responses(&self) -> Vec<PlainMessage> {
            self.received_responses.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl EventSubscriber for TestTrustPingSubscriber {
        async fn handle_event(&self, event: NodeEvent) {
            if let NodeEvent::PlainMessageSent {
                message,
                from: _,
                to: _,
            } = event
            {
                if let Ok(plain_message) = serde_json::from_value::<PlainMessage>(message) {
                    if plain_message.type_ == TrustPingResponse::message_type() {
                        self.received_responses.lock().unwrap().push(plain_message);
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn test_trust_ping_processor_generates_response() {
        // Create an event bus
        let event_bus = Arc::new(EventBus::new());

        // Create a test subscriber to capture response events
        let subscriber = Arc::new(TestTrustPingSubscriber::new());
        event_bus.subscribe(subscriber.clone()).await;

        // Create a Trust Ping processor with the event bus
        let processor = TrustPingProcessor::with_event_bus(event_bus.clone());

        // Create a Trust Ping message that requests a response
        let ping = TrustPing::new().response_requested(true);
        let ping_message = PlainMessage {
            id: "ping-test-123".to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: TrustPing::message_type().to_string(),
            body: serde_json::to_value(&ping).unwrap(),
            from: "did:example:sender".to_string(),
            to: vec!["did:example:recipient".to_string()],
            thid: None,
            pthid: None,
            extra_headers: HashMap::new(),
            attachments: None,
            created_time: Some(chrono::Utc::now().timestamp_millis() as u64),
            expires_time: None,
            from_prior: None,
        };

        // Process the ping message
        let result = processor.process_incoming(ping_message.clone()).await;

        // Verify the message was passed through unchanged
        assert!(result.is_ok());
        let processed_message = result.unwrap();
        assert!(processed_message.is_some());
        let processed_message = processed_message.unwrap();
        assert_eq!(processed_message.id, ping_message.id);

        // Give the event system time to process
        sleep(Duration::from_millis(10)).await;

        // Verify a response was published to the event bus
        let responses = subscriber.get_responses();
        assert_eq!(responses.len(), 1);

        let response_message = &responses[0];
        assert_eq!(response_message.type_, TrustPingResponse::message_type());
        assert_eq!(response_message.from, "did:example:recipient");
        assert_eq!(response_message.to, vec!["did:example:sender"]);
        assert_eq!(response_message.thid, Some("ping-test-123".to_string()));

        // Verify the response body
        let response_body: TrustPingResponse =
            serde_json::from_value(response_message.body.clone()).unwrap();
        assert_eq!(response_body.thread_id, "ping-test-123");
        assert_eq!(response_body.comment, Some("Pong!".to_string()));
    }

    #[tokio::test]
    async fn test_trust_ping_processor_no_response_requested() {
        // Create an event bus
        let event_bus = Arc::new(EventBus::new());

        // Create a test subscriber to capture response events
        let subscriber = Arc::new(TestTrustPingSubscriber::new());
        event_bus.subscribe(subscriber.clone()).await;

        // Create a Trust Ping processor with the event bus
        let processor = TrustPingProcessor::with_event_bus(event_bus.clone());

        // Create a Trust Ping message that does NOT request a response
        let ping = TrustPing::new().response_requested(false);
        let ping_message = PlainMessage {
            id: "ping-no-response-123".to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: TrustPing::message_type().to_string(),
            body: serde_json::to_value(&ping).unwrap(),
            from: "did:example:sender".to_string(),
            to: vec!["did:example:recipient".to_string()],
            thid: None,
            pthid: None,
            extra_headers: HashMap::new(),
            attachments: None,
            created_time: Some(chrono::Utc::now().timestamp_millis() as u64),
            expires_time: None,
            from_prior: None,
        };

        // Process the ping message
        let result = processor.process_incoming(ping_message.clone()).await;

        // Verify the message was passed through unchanged
        assert!(result.is_ok());
        let processed_message = result.unwrap();
        assert!(processed_message.is_some());
        let processed_message = processed_message.unwrap();
        assert_eq!(processed_message.id, ping_message.id);

        // Give the event system time to process
        sleep(Duration::from_millis(10)).await;

        // Verify NO response was published to the event bus
        let responses = subscriber.get_responses();
        assert_eq!(responses.len(), 0);
    }

    #[tokio::test]
    async fn test_trust_ping_processor_without_event_bus() {
        // Create a Trust Ping processor WITHOUT an event bus
        let processor = TrustPingProcessor::new();

        // Create a Trust Ping message that requests a response
        let ping = TrustPing::new().response_requested(true);
        let ping_message = PlainMessage {
            id: "ping-no-bus-123".to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: TrustPing::message_type().to_string(),
            body: serde_json::to_value(&ping).unwrap(),
            from: "did:example:sender".to_string(),
            to: vec!["did:example:recipient".to_string()],
            thid: None,
            pthid: None,
            extra_headers: HashMap::new(),
            attachments: None,
            created_time: Some(chrono::Utc::now().timestamp_millis() as u64),
            expires_time: None,
            from_prior: None,
        };

        // Process the ping message - should still work, just no response sent
        let result = processor.process_incoming(ping_message.clone()).await;

        // Verify the message was passed through unchanged
        assert!(result.is_ok());
        let processed_message = result.unwrap();
        assert!(processed_message.is_some());
        let processed_message = processed_message.unwrap();
        assert_eq!(processed_message.id, ping_message.id);
    }

    #[tokio::test]
    async fn test_trust_ping_processor_non_trust_ping_message() {
        // Create an event bus
        let event_bus = Arc::new(EventBus::new());

        // Create a test subscriber to capture response events
        let subscriber = Arc::new(TestTrustPingSubscriber::new());
        event_bus.subscribe(subscriber.clone()).await;

        // Create a Trust Ping processor with the event bus
        let processor = TrustPingProcessor::with_event_bus(event_bus.clone());

        // Create a non-Trust Ping message
        let normal_message = PlainMessage {
            id: "normal-message-123".to_string(),
            typ: "application/didcomm-plain+json".to_string(),
            type_: "https://example.com/normal-message".to_string(),
            body: serde_json::json!({"content": "Hello, world!"}),
            from: "did:example:sender".to_string(),
            to: vec!["did:example:recipient".to_string()],
            thid: None,
            pthid: None,
            extra_headers: HashMap::new(),
            attachments: None,
            created_time: Some(chrono::Utc::now().timestamp_millis() as u64),
            expires_time: None,
            from_prior: None,
        };

        // Process the normal message
        let result = processor.process_incoming(normal_message.clone()).await;

        // Verify the message was passed through unchanged
        assert!(result.is_ok());
        let processed_message = result.unwrap();
        assert!(processed_message.is_some());
        let processed_message = processed_message.unwrap();
        assert_eq!(processed_message.id, normal_message.id);

        // Give the event system time to process
        sleep(Duration::from_millis(10)).await;

        // Verify NO response was published to the event bus
        let responses = subscriber.get_responses();
        assert_eq!(responses.len(), 0);
    }
}
