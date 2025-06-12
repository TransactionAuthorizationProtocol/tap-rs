//! JSON schemas for tool parameters

use serde_json::{json, Value};

/// Schema for create_agent tool
pub fn create_agent_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "label": {
                "type": "string",
                "description": "Optional human-friendly label for the agent key"
            }
        },
        "additionalProperties": false
    })
}

/// Schema for list_agents tool
pub fn list_agents_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
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
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent that will sign and send this message"
            },
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
        "required": ["agent_did", "asset", "amount", "originator", "beneficiary"],
        "additionalProperties": false
    })
}

/// Schema for authorize tool
pub fn authorize_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent that will sign and send this message"
            },
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
        "required": ["agent_did", "transaction_id"],
        "additionalProperties": false
    })
}

/// Schema for reject tool
pub fn reject_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent that will sign and send this message"
            },
            "transaction_id": {
                "type": "string",
                "description": "Transaction ID to reject"
            },
            "reason": {
                "type": "string",
                "description": "Reason for rejection"
            }
        },
        "required": ["agent_did", "transaction_id", "reason"],
        "additionalProperties": false
    })
}

/// Schema for cancel tool
pub fn cancel_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent that will sign and send this message"
            },
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
        "required": ["agent_did", "transaction_id", "by"],
        "additionalProperties": false
    })
}

/// Schema for settle tool
pub fn settle_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent that will sign and send this message"
            },
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
        "required": ["agent_did", "transaction_id", "settlement_id"],
        "additionalProperties": false
    })
}

/// Schema for list_transactions tool
pub fn list_transactions_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent whose transactions to list"
            },
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
        "required": ["agent_did"],
        "additionalProperties": false
    })
}

/// Schema for list_customers tool
pub fn list_customers_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent whose customers to list"
            },
            "limit": {
                "type": "number",
                "description": "Maximum number of customers to return",
                "default": 50,
                "minimum": 1,
                "maximum": 1000
            },
            "offset": {
                "type": "number",
                "description": "Number of customers to skip for pagination",
                "default": 0,
                "minimum": 0
            }
        },
        "required": ["agent_did"],
        "additionalProperties": false
    })
}

/// Schema for list_connections tool
pub fn list_connections_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "party_id": {
                "type": "string",
                "description": "The DID of the party whose connections to list"
            },
            "limit": {
                "type": "number",
                "description": "Maximum number of connections to return",
                "default": 50,
                "minimum": 1,
                "maximum": 1000
            },
            "offset": {
                "type": "number",
                "description": "Number of connections to skip for pagination",
                "default": 0,
                "minimum": 0
            }
        },
        "required": ["party_id"],
        "additionalProperties": false
    })
}

/// Schema for get_customer_details tool
pub fn get_customer_details_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent managing the customer"
            },
            "customer_id": {
                "type": "string",
                "description": "The ID of the customer to retrieve"
            }
        },
        "required": ["agent_did", "customer_id"],
        "additionalProperties": false
    })
}

/// Schema for generate_ivms101 tool
pub fn generate_ivms101_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent managing the customer"
            },
            "customer_id": {
                "type": "string",
                "description": "The ID of the customer to generate IVMS101 data for"
            }
        },
        "required": ["agent_did", "customer_id"],
        "additionalProperties": false
    })
}

/// Schema for update_customer_profile tool
pub fn update_customer_profile_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent managing the customer"
            },
            "customer_id": {
                "type": "string",
                "description": "The ID of the customer to update"
            },
            "profile_data": {
                "type": "object",
                "description": "Schema.org profile data to update/add (e.g., givenName, familyName, addressCountry)"
            }
        },
        "required": ["agent_did", "customer_id", "profile_data"],
        "additionalProperties": false
    })
}

/// Schema for update_customer_from_ivms101 tool
pub fn update_customer_from_ivms101_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent managing the customer"
            },
            "customer_id": {
                "type": "string",
                "description": "The ID of the customer to update"
            },
            "ivms101_data": {
                "type": "object",
                "description": "IVMS101 data containing naturalPerson or legalPerson information"
            }
        },
        "required": ["agent_did", "customer_id", "ivms101_data"],
        "additionalProperties": false
    })
}

/// Schema for revert tool
pub fn revert_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent that will sign and send this message"
            },
            "transaction_id": {
                "type": "string",
                "description": "ID of the transaction to revert"
            },
            "settlement_address": {
                "type": "string",
                "description": "Settlement address in CAIP-10 format to return the funds to"
            },
            "reason": {
                "type": "string",
                "description": "Reason for the reversal request"
            }
        },
        "required": ["agent_did", "transaction_id", "settlement_address", "reason"],
        "additionalProperties": false
    })
}

/// Schema for payment tool
pub fn payment_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent that will sign and send this message"
            },
            "amount": {
                "type": "string",
                "description": "Payment amount as decimal string"
            },
            "asset": {
                "type": "string",
                "description": "CAIP-19 asset identifier (e.g., 'eip155:1/erc20:0x...')"
            },
            "payer": {
                "type": "object",
                "properties": {
                    "@id": {
                        "type": "string",
                        "description": "DID or identifier of the payer (optional - if not provided, message is sent to payee)"
                    }
                },
                "required": ["@id"],
                "additionalProperties": false
            },
            "payee": {
                "type": "object",
                "properties": {
                    "@id": {
                        "type": "string",
                        "description": "DID or identifier of the payee"
                    }
                },
                "required": ["@id"],
                "additionalProperties": false
            },
            "memo": {
                "type": "string",
                "description": "Optional payment memo or description"
            },
            "metadata": {
                "type": "object",
                "description": "Optional additional metadata"
            }
        },
        "required": ["agent_did", "amount", "asset", "payee"],
        "additionalProperties": false
    })
}

/// Schema for connect tool
pub fn connect_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent that will sign and send this message"
            },
            "recipient_did": {
                "type": "string",
                "description": "The DID of the party to connect with"
            },
            "constraints": {
                "type": "object",
                "properties": {
                    "transaction_limits": {
                        "type": "object",
                        "properties": {
                            "daily_limit": {
                                "type": "string",
                                "description": "Maximum daily transaction amount"
                            },
                            "monthly_limit": {
                                "type": "string",
                                "description": "Maximum monthly transaction amount"
                            },
                            "per_transaction_limit": {
                                "type": "string",
                                "description": "Maximum per-transaction amount"
                            }
                        },
                        "additionalProperties": false
                    },
                    "required_fields": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "List of required data fields"
                    },
                    "allowed_assets": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "List of allowed CAIP-19 asset identifiers"
                    }
                },
                "additionalProperties": false
            },
            "metadata": {
                "type": "object",
                "description": "Optional additional metadata"
            }
        },
        "required": ["agent_did", "recipient_did"],
        "additionalProperties": false
    })
}

/// Schema for authorization_required tool
pub fn authorization_required_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent that will sign and send this message"
            },
            "recipient_did": {
                "type": "string",
                "description": "The DID of the party that needs to authorize"
            },
            "connection_id": {
                "type": "string",
                "description": "The connection ID requiring authorization"
            },
            "authorization_url": {
                "type": "string",
                "description": "URL where authorization can be completed"
            }
        },
        "required": ["agent_did", "recipient_did", "connection_id", "authorization_url"],
        "additionalProperties": false
    })
}

/// Schema for out_of_band tool
pub fn out_of_band_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent creating the invitation"
            },
            "goal_code": {
                "type": "string",
                "description": "Goal code for the invitation (e.g., 'tap.connect', 'tap.transfer', 'tap.payment')"
            },
            "goal": {
                "type": "string",
                "description": "Human-readable description of the goal"
            },
            "attachments": {
                "type": "array",
                "items": {
                    "type": "object"
                },
                "description": "Optional attachments to include with the invitation"
            }
        },
        "required": ["agent_did", "goal_code"],
        "additionalProperties": false
    })
}

/// Schema for update_party tool
pub fn update_party_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent that will sign and send this message"
            },
            "recipient_did": {
                "type": "string",
                "description": "The DID of the party to receive the update"
            },
            "party_id": {
                "type": "string",
                "description": "The ID of the party being updated"
            },
            "metadata": {
                "type": "object",
                "description": "Updated metadata for the party"
            }
        },
        "required": ["agent_did", "recipient_did", "party_id"],
        "additionalProperties": false
    })
}

/// Schema for confirm_relationship tool
pub fn confirm_relationship_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent that will sign and send this message"
            },
            "recipient_did": {
                "type": "string",
                "description": "The DID of the party to receive the confirmation"
            },
            "relationship_type": {
                "type": "string",
                "description": "Type of relationship (e.g., 'controls', 'owns', 'manages')"
            },
            "subject": {
                "type": "string",
                "description": "Subject of the relationship (e.g., DID or identifier)"
            },
            "proof": {
                "type": "object",
                "description": "Optional proof of the relationship"
            }
        },
        "required": ["agent_did", "recipient_did", "relationship_type", "subject"],
        "additionalProperties": false
    })
}

/// Schema for update_policies tool
pub fn update_policies_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent that will sign and send this message"
            },
            "recipient_did": {
                "type": "string",
                "description": "The DID of the party to receive the policy update"
            },
            "policies": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "@type": {
                            "type": "string",
                            "description": "Policy type (e.g., 'RequireAuthorization', 'RequirePresentation')"
                        }
                    },
                    "required": ["@type"],
                    "additionalProperties": true
                },
                "description": "List of policies to update"
            }
        },
        "required": ["agent_did", "recipient_did", "policies"],
        "additionalProperties": false
    })
}

/// Schema for add_agents tool
pub fn add_agents_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent that will sign and send this message"
            },
            "transaction_id": {
                "type": "string",
                "description": "ID of the transaction to add agents to"
            },
            "agents": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "@id": {
                            "type": "string",
                            "description": "Agent DID"
                        },
                        "role": {
                            "type": "string",
                            "description": "Agent role (e.g., 'SettlementAddress', 'ComplianceOfficer')"
                        },
                        "for": {
                            "type": "string",
                            "description": "DID of the party this agent represents"
                        }
                    },
                    "required": ["@id", "role", "for"],
                    "additionalProperties": false
                },
                "description": "List of agents to add"
            }
        },
        "required": ["agent_did", "transaction_id", "agents"],
        "additionalProperties": false
    })
}

/// Schema for remove_agent tool
pub fn remove_agent_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent that will sign and send this message"
            },
            "transaction_id": {
                "type": "string",
                "description": "ID of the transaction to remove agent from"
            },
            "agent_to_remove": {
                "type": "string",
                "description": "DID of the agent to remove"
            }
        },
        "required": ["agent_did", "transaction_id", "agent_to_remove"],
        "additionalProperties": false
    })
}

/// Schema for replace_agent tool
pub fn replace_agent_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent that will sign and send this message"
            },
            "transaction_id": {
                "type": "string",
                "description": "ID of the transaction to replace agent in"
            },
            "original_agent": {
                "type": "string",
                "description": "DID of the agent to replace"
            },
            "new_agent": {
                "type": "object",
                "properties": {
                    "@id": {
                        "type": "string",
                        "description": "New agent DID"
                    },
                    "role": {
                        "type": "string",
                        "description": "Agent role"
                    },
                    "for": {
                        "type": "string",
                        "description": "DID of the party this agent represents"
                    }
                },
                "required": ["@id", "role", "for"],
                "additionalProperties": false
            }
        },
        "required": ["agent_did", "transaction_id", "original_agent", "new_agent"],
        "additionalProperties": false
    })
}

/// Schema for request_presentation tool
pub fn request_presentation_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent that will sign and send this message"
            },
            "recipient_did": {
                "type": "string",
                "description": "The DID of the party to request presentation from"
            },
            "requested_attributes": {
                "type": "array",
                "items": {
                    "type": "string"
                },
                "description": "List of attributes to request (e.g., ['name', 'address', 'dateOfBirth'])"
            },
            "purpose": {
                "type": "string",
                "description": "Purpose of the request"
            },
            "challenge": {
                "type": "string",
                "description": "Optional challenge for the presentation"
            }
        },
        "required": ["agent_did", "recipient_did", "requested_attributes"],
        "additionalProperties": false
    })
}

/// Schema for presentation tool
pub fn presentation_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "agent_did": {
                "type": "string",
                "description": "The DID of the agent that will sign and send this message"
            },
            "recipient_did": {
                "type": "string",
                "description": "The DID of the party to send presentation to"
            },
            "thread_id": {
                "type": "string",
                "description": "Thread ID from the presentation request"
            },
            "presented_attributes": {
                "type": "object",
                "description": "JSON object with the attributes being presented"
            },
            "proof": {
                "type": "object",
                "description": "Optional proof of the presented attributes"
            }
        },
        "required": ["agent_did", "recipient_did", "presented_attributes"],
        "additionalProperties": false
    })
}
