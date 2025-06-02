-- Create transaction_agents table for tracking agent statuses per transaction
CREATE TABLE IF NOT EXISTS transaction_agents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transaction_id INTEGER NOT NULL,
    agent_did TEXT NOT NULL,
    agent_role TEXT NOT NULL CHECK (agent_role IN ('sender', 'receiver', 'compliance', 'other')),
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'authorized', 'rejected', 'cancelled')),
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    FOREIGN KEY (transaction_id) REFERENCES transactions(id) ON DELETE CASCADE,
    UNIQUE(transaction_id, agent_did)
);

-- Create indexes for efficient queries
CREATE INDEX idx_transaction_agents_transaction_id ON transaction_agents(transaction_id);
CREATE INDEX idx_transaction_agents_agent_did ON transaction_agents(agent_did);
CREATE INDEX idx_transaction_agents_status ON transaction_agents(status);
CREATE INDEX idx_transaction_agents_role ON transaction_agents(agent_role);

-- Create trigger to update updated_at timestamp
CREATE TRIGGER set_transaction_agents_updated_at
AFTER UPDATE ON transaction_agents
BEGIN
    UPDATE transaction_agents SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = NEW.id;
END;