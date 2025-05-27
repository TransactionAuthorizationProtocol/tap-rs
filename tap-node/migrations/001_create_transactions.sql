-- Create transactions table
CREATE TABLE IF NOT EXISTS transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    type TEXT NOT NULL CHECK (type IN ('transfer', 'payment')),
    reference_id TEXT NOT NULL UNIQUE,
    from_did TEXT,
    to_did TEXT,
    thread_id TEXT,
    message_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'confirmed', 'failed', 'cancelled', 'reverted')),
    message_json JSON NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Create indexes for common queries
CREATE INDEX idx_transactions_status ON transactions(status);
CREATE INDEX idx_transactions_type ON transactions(type);
CREATE INDEX idx_transactions_from_did ON transactions(from_did);
CREATE INDEX idx_transactions_to_did ON transactions(to_did);
CREATE INDEX idx_transactions_thread_id ON transactions(thread_id);
CREATE INDEX idx_transactions_created_at ON transactions(created_at);

-- Create trigger to update updated_at timestamp
CREATE TRIGGER set_updated_at
AFTER UPDATE ON transactions
BEGIN
    UPDATE transactions SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = NEW.id;
END;