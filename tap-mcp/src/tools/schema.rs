//! JSON schemas for tool parameters

use serde_json::{json, Value};

/// Schema for create_agent tool
pub fn create_agent_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "@id": {
                "type": "string",
                "description": "Agent DID identifier"
            },
            "role": {
                "type": "string",
                "description": "Agent role (e.g., 'SettlementAddress', 'Exchange')"
            },
            "for": {
                "type": "string",
                "description": "DID of party the agent acts for"
            },
            "policies": {
                "type": "array",
                "description": "Optional agent policies (TAIP-7)",
                "items": {
                    "type": "object"
                }
            },
            "metadata": {
                "type": "object",
                "description": "Optional additional metadata"
            }
        },
        "required": ["@id", "role", "for"],
        "additionalProperties": false
    })
}

/// Schema for list_agents tool
pub fn list_agents_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "filter": {
                "type": "object",
                "properties": {
                    "role": {
                        "type": "string",
                        "description": "Filter by agent role"
                    },
                    "for_party": {
                        "type": "string",
                        "description": "Filter by party DID"
                    }
                },
                "additionalProperties": false
            },
            "limit": {
                "type": "number",
                "description": "Maximum number of agents to return",
                "default": 50,
                "minimum": 1,
                "maximum": 1000
            },
            "offset": {
                "type": "number",
                "description": "Number of agents to skip for pagination",
                "default": 0,
                "minimum": 0
            }
        },
        "additionalProperties": false
    })
}

/// Schema for create_transfer tool
pub fn create_transfer_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "asset": {
                "type": "string",
                "description": "CAIP-19 asset identifier (e.g., 'eip155:1/erc20:0x...')"
            },
            "amount": {
                "type": "string",
                "description": "Transfer amount as decimal string"
            },
            "originator": {
                "type": "object",
                "properties": {
                    "@id": {
                        "type": "string",
                        "description": "DID of the originator"
                    },
                    "metadata": {
                        "type": "object",
                        "description": "Optional additional party metadata"
                    }
                },
                "required": ["@id"],
                "additionalProperties": false
            },
            "beneficiary": {
                "type": "object",
                "properties": {
                    "@id": {
                        "type": "string",
                        "description": "DID of the beneficiary"
                    },
                    "metadata": {
                        "type": "object",
                        "description": "Optional additional party metadata"
                    }
                },
                "required": ["@id"],
                "additionalProperties": false
            },
            "agents": {
                "type": "array",
                "description": "List of agents involved in the transaction",
                "items": {
                    "type": "object",
                    "properties": {
                        "@id": {
                            "type": "string",
                            "description": "Agent DID"
                        },
                        "role": {
                            "type": "string",
                            "description": "Agent role"
                        },
                        "for": {
                            "type": "string",
                            "description": "DID of party agent acts for"
                        }
                    },
                    "required": ["@id", "role", "for"],
                    "additionalProperties": false
                }
            },
            "memo": {
                "type": "string",
                "description": "Optional transaction memo"
            },
            "metadata": {
                "type": "object",
                "description": "Optional additional metadata"
            }
        },
        "required": ["asset", "amount", "originator", "beneficiary"],
        "additionalProperties": false
    })
}

/// Schema for authorize tool
pub fn authorize_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "transaction_id": {
                "type": "string",
                "description": "Transaction ID to authorize"
            },
            "settlement_address": {
                "type": "string",
                "description": "Optional CAIP-10 format settlement address"
            },
            "expiry": {
                "type": "string",
                "description": "Optional ISO 8601 expiry time"
            }
        },
        "required": ["transaction_id"],
        "additionalProperties": false
    })
}

/// Schema for reject tool
pub fn reject_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "transaction_id": {
                "type": "string",
                "description": "Transaction ID to reject"
            },
            "reason": {
                "type": "string",
                "description": "Reason for rejection"
            }
        },
        "required": ["transaction_id", "reason"],
        "additionalProperties": false
    })
}

/// Schema for cancel tool
pub fn cancel_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "transaction_id": {
                "type": "string",
                "description": "Transaction ID to cancel"
            },
            "by": {
                "type": "string",
                "description": "Party requesting cancellation"
            },
            "reason": {
                "type": "string",
                "description": "Optional reason for cancellation"
            }
        },
        "required": ["transaction_id", "by"],
        "additionalProperties": false
    })
}

/// Schema for settle tool
pub fn settle_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "transaction_id": {
                "type": "string",
                "description": "Transaction ID to settle"
            },
            "settlement_id": {
                "type": "string",
                "description": "CAIP-220 settlement identifier"
            },
            "amount": {
                "type": "string",
                "description": "Optional amount settled"
            }
        },
        "required": ["transaction_id", "settlement_id"],
        "additionalProperties": false
    })
}

/// Schema for list_transactions tool
pub fn list_transactions_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "filter": {
                "type": "object",
                "properties": {
                    "message_type": {
                        "type": "string",
                        "description": "Filter by TAP message type"
                    },
                    "thread_id": {
                        "type": "string",
                        "description": "Filter by thread ID"
                    },
                    "from_did": {
                        "type": "string",
                        "description": "Filter by sender DID"
                    },
                    "to_did": {
                        "type": "string",
                        "description": "Filter by recipient DID"
                    },
                    "date_from": {
                        "type": "string",
                        "description": "Filter by start date (ISO 8601)"
                    },
                    "date_to": {
                        "type": "string",
                        "description": "Filter by end date (ISO 8601)"
                    }
                },
                "additionalProperties": false
            },
            "sort": {
                "type": "object",
                "properties": {
                    "field": {
                        "type": "string",
                        "enum": ["created_time", "id"],
                        "description": "Field to sort by"
                    },
                    "order": {
                        "type": "string",
                        "enum": ["asc", "desc"],
                        "description": "Sort order"
                    }
                },
                "additionalProperties": false
            },
            "limit": {
                "type": "number",
                "description": "Maximum number of transactions to return",
                "default": 50,
                "minimum": 1,
                "maximum": 1000
            },
            "offset": {
                "type": "number",
                "description": "Number of transactions to skip for pagination",
                "default": 0,
                "minimum": 0
            }
        },
        "additionalProperties": false
    })
}
