-- Decision log for tracking external decision requests and their resolution.
-- Each row represents a decision point that the FSM produced and that an
-- external system (compliance engine, AI agent, human operator) must resolve.

CREATE TABLE IF NOT EXISTS decision_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    transaction_id TEXT NOT NULL,
    agent_did TEXT NOT NULL,
    decision_type TEXT NOT NULL CHECK (decision_type IN (
        'authorization_required',
        'policy_satisfaction_required',
        'settlement_required'
    )),
    context_json TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN (
        'pending',
        'delivered',
        'resolved',
        'expired'
    )),
    resolution TEXT,
    resolution_detail TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    delivered_at TEXT,
    resolved_at TEXT
);

CREATE INDEX idx_decision_log_transaction_id ON decision_log(transaction_id);
CREATE INDEX idx_decision_log_agent_did ON decision_log(agent_did);
CREATE INDEX idx_decision_log_status ON decision_log(status);
CREATE INDEX idx_decision_log_decision_type ON decision_log(decision_type);
CREATE INDEX idx_decision_log_created_at ON decision_log(created_at);
CREATE INDEX idx_decision_log_status_created ON decision_log(status, created_at);
