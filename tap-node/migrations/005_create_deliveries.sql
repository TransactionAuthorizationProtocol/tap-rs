-- Create deliveries table for tracking message delivery status
CREATE TABLE deliveries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id TEXT NOT NULL,
    message_text TEXT NOT NULL,
    recipient_did TEXT NOT NULL,
    delivery_url TEXT,
    delivery_type TEXT NOT NULL DEFAULT 'https' CHECK (delivery_type IN ('https', 'internal', 'return_path', 'pickup')),
    status TEXT NOT NULL CHECK (status IN ('pending', 'success', 'failed')) DEFAULT 'pending',
    retry_count INTEGER NOT NULL DEFAULT 0,
    last_http_status_code INTEGER,
    error_message TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    delivered_at TIMESTAMP,
    
    -- Create indexes for efficient querying
    FOREIGN KEY (message_id) REFERENCES messages(message_id)
);

-- Index for querying deliveries by message
CREATE INDEX idx_deliveries_message_id ON deliveries(message_id);

-- Index for querying deliveries by recipient
CREATE INDEX idx_deliveries_recipient_did ON deliveries(recipient_did);

-- Index for querying deliveries by status
CREATE INDEX idx_deliveries_status ON deliveries(status);

-- Index for querying deliveries by delivery type
CREATE INDEX idx_deliveries_type ON deliveries(delivery_type);

-- Index for querying pending deliveries for retry processing
CREATE INDEX idx_deliveries_pending_retry ON deliveries(status, retry_count, created_at);

-- Composite index for querying pending deliveries by type
CREATE INDEX idx_deliveries_type_status_retry ON deliveries(delivery_type, status, retry_count);

-- Trigger to automatically update updated_at timestamp
CREATE TRIGGER update_deliveries_updated_at
    AFTER UPDATE ON deliveries
    FOR EACH ROW
BEGIN
    UPDATE deliveries SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;