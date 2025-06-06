-- Create received table for storing raw incoming messages
CREATE TABLE IF NOT EXISTS received (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    -- Unique message ID from the message itself (if available)
    message_id TEXT,
    -- Raw message content (JWE, JWS, or plain JSON)
    raw_message TEXT NOT NULL,
    -- Source of the message
    source_type TEXT NOT NULL CHECK (source_type IN ('https', 'internal', 'websocket', 'return_path', 'pickup')),
    -- Source identifier (URL, agent DID, etc.)
    source_identifier TEXT,
    -- Processing status
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'processed', 'failed')),
    -- Error message if processing failed
    error_message TEXT,
    -- Timestamps
    received_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    processed_at TEXT,
    -- Foreign key to messages table (populated after successful processing)
    processed_message_id TEXT
);

-- Create indexes for efficient querying
CREATE INDEX idx_received_message_id ON received(message_id);
CREATE INDEX idx_received_status ON received(status);
CREATE INDEX idx_received_source_type ON received(source_type);
CREATE INDEX idx_received_received_at ON received(received_at);
CREATE INDEX idx_received_processed_message_id ON received(processed_message_id);

-- Remove raw_message column from messages table if it exists
-- SQLite supports DROP COLUMN since version 3.35.0
ALTER TABLE messages DROP COLUMN IF EXISTS raw_message;