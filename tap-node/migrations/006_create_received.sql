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
CREATE INDEX IF NOT EXISTS idx_received_message_id ON received(message_id);
CREATE INDEX IF NOT EXISTS idx_received_status ON received(status);
CREATE INDEX IF NOT EXISTS idx_received_source_type ON received(source_type);
CREATE INDEX IF NOT EXISTS idx_received_received_at ON received(received_at);
CREATE INDEX IF NOT EXISTS idx_received_processed_message_id ON received(processed_message_id);

-- Remove raw_message column from messages table
-- SQLite 3.35.0+ supports DROP COLUMN (without IF EXISTS)
-- We'll use a safe approach that checks if the column exists first
-- by attempting to create a new table and copying data

-- Create temporary table without raw_message column
CREATE TEMPORARY TABLE IF NOT EXISTS messages_new_temp (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id TEXT NOT NULL UNIQUE,
    message_type TEXT NOT NULL,
    from_did TEXT,
    to_did TEXT,
    thread_id TEXT,
    parent_thread_id TEXT,
    direction TEXT NOT NULL CHECK (direction IN ('incoming', 'outgoing')),
    message_json JSONB NOT NULL,
    status TEXT DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'rejected')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Try to copy data from messages table (this will work whether raw_message exists or not)
INSERT OR IGNORE INTO messages_new_temp (id, message_id, message_type, from_did, to_did, thread_id, parent_thread_id, direction, message_json, status, created_at)
SELECT id, message_id, message_type, from_did, to_did, thread_id, parent_thread_id, direction, message_json, status, created_at
FROM messages;

-- Drop and recreate the messages table
DROP TABLE IF EXISTS messages;
CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    message_id TEXT NOT NULL UNIQUE,
    message_type TEXT NOT NULL,
    from_did TEXT,
    to_did TEXT,
    thread_id TEXT,
    parent_thread_id TEXT,
    direction TEXT NOT NULL CHECK (direction IN ('incoming', 'outgoing')),
    message_json JSONB NOT NULL,
    status TEXT DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'rejected')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Copy data back from temporary table
INSERT INTO messages SELECT * FROM messages_new_temp;
DROP TABLE messages_new_temp;

-- Recreate all the indexes
CREATE INDEX IF NOT EXISTS idx_messages_message_id ON messages(message_id);
CREATE INDEX IF NOT EXISTS idx_messages_message_type ON messages(message_type);
CREATE INDEX IF NOT EXISTS idx_messages_from_did ON messages(from_did);
CREATE INDEX IF NOT EXISTS idx_messages_to_did ON messages(to_did);
CREATE INDEX IF NOT EXISTS idx_messages_thread_id ON messages(thread_id);
CREATE INDEX IF NOT EXISTS idx_messages_direction ON messages(direction);
CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at);
CREATE INDEX IF NOT EXISTS idx_messages_status ON messages(status);