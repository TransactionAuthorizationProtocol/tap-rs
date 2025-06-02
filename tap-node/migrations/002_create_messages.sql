-- Create messages table for audit trail of all incoming messages
CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id TEXT NOT NULL UNIQUE,
    message_type TEXT NOT NULL,
    from_did TEXT,
    to_did TEXT,
    thread_id TEXT,
    parent_thread_id TEXT,
    direction TEXT NOT NULL CHECK (direction IN ('incoming', 'outgoing')),
    message_json JSONB NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Create indexes for common queries
CREATE INDEX idx_messages_message_id ON messages(message_id);
CREATE INDEX idx_messages_message_type ON messages(message_type);
CREATE INDEX idx_messages_from_did ON messages(from_did);
CREATE INDEX idx_messages_to_did ON messages(to_did);
CREATE INDEX idx_messages_thread_id ON messages(thread_id);
CREATE INDEX idx_messages_direction ON messages(direction);
CREATE INDEX idx_messages_created_at ON messages(created_at);
