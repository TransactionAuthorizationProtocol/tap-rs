//! Transaction Finite State Machine (FSM)
//!
//! Formal state machine modeling the lifecycle of a TAP transaction from
//! initiation through authorization to settlement. Each state transition
//! is driven by an incoming TAP message and may require an external
//! decision before the node takes action.
//!
//! # States
//!
//! ```text
//!                          ┌──────────────────────────────────────────┐
//!                          │         Transaction Lifecycle            │
//!                          └──────────────────────────────────────────┘
//!
//!   ┌─────────┐  Transfer/   ┌──────────────┐  UpdatePolicies/  ┌─────────────────┐
//!   │         │  Payment     │              │  RequestPresent.  │                 │
//!   │  (none) │─────────────▶│   Received    │─────────────────▶│ PolicyRequired  │
//!   │         │              │              │                   │                 │
//!   └─────────┘              └──────┬───────┘                   └────────┬────────┘
//!                                   │                                    │
//!                              ┌────┴─────┐                    Presentation
//!                              │ DECISION │                    received
//!                              │ Authorize│                         │
//!                              │ Reject   │◀────────────────────────┘
//!                              │ Cancel   │
//!                              └────┬─────┘
//!                    ┌──────────────┼──────────────┐
//!                    │              │              │
//!                Authorize       Reject         Cancel
//!                    │              │              │
//!                    ▼              ▼              ▼
//!            ┌──────────────┐ ┌──────────┐ ┌───────────┐
//!            │  Authorized  │ │ Rejected │ │ Cancelled │
//!            │ (per agent)  │ │          │ │           │
//!            └──────┬───────┘ └──────────┘ └───────────┘
//!                   │
//!            all agents authorized?
//!                   │ yes
//!                   ▼
//!            ┌──────────────────┐
//!            │ ReadyToSettle    │
//!            │                  │
//!            └────────┬─────────┘
//!                     │
//!                ┌────┴─────┐
//!                │ DECISION │
//!                │ Settle   │
//!                │ Cancel   │
//!                └────┬─────┘
//!                     │
//!                  Settle
//!                     │
//!                     ▼
//!              ┌─────────────┐
//!              │  Settled    │
//!              └──────┬──────┘
//!                     │
//!                  Revert?
//!                     │
//!                     ▼
//!              ┌─────────────┐
//!              │  Reverted   │
//!              └─────────────┘
//! ```
//!
//! # Decision Points
//!
//! The FSM identifies two categories of transitions:
//!
//! - **Automatic**: The node processes the message and moves to the next state
//!   with no external input (e.g., storing a transaction, recording an authorization).
//!
//! - **Decision Required**: The transition produces a [`Decision`] that an
//!   external system must resolve before the node takes further action. For
//!   example, when a Transfer arrives the node must decide whether to
//!   Authorize, Reject, or request more information via policies.
//!
//! # Per-Agent vs Per-Transaction State
//!
//! A transaction has a single top-level [`TransactionState`], but also tracks
//! per-agent authorization status via [`AgentState`]. The transaction advances
//! to `ReadyToSettle` only when **all** agents reach `Authorized`.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Transaction States
// ---------------------------------------------------------------------------

/// Top-level state of a TAP transaction.
///
/// These states represent the full lifecycle from initiation to terminal
/// states. The FSM enforces that only valid transitions occur.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransactionState {
    /// Transaction initiated — a Transfer or Payment has been received and
    /// stored. The node must now decide how to respond (authorize, reject,
    /// request more info, or wait for external input).
    Received,

    /// One or more counterparty policies must be satisfied before
    /// authorization can proceed. The node is waiting for the external
    /// system to gather and submit the required presentations or proofs.
    PolicyRequired,

    /// At least one agent has authorized but not all required agents have
    /// done so yet. The transaction is waiting for remaining authorizations.
    PartiallyAuthorized,

    /// All required agents have authorized. The originator may now settle
    /// the transaction on-chain. This is a decision point — the node must
    /// decide whether to proceed with settlement.
    ReadyToSettle,

    /// The originator has sent a Settle message (with an on-chain
    /// transaction reference). The transaction is considered complete.
    Settled,

    /// An agent has rejected the transaction. Terminal state.
    Rejected,

    /// A party has cancelled the transaction. Terminal state.
    Cancelled,

    /// A previously settled transaction has been reverted. Terminal state.
    Reverted,
}

impl TransactionState {
    /// Returns true if this is a terminal state (no further transitions).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TransactionState::Rejected | TransactionState::Cancelled | TransactionState::Reverted
        )
    }

    /// Returns true if this state requires an external decision before
    /// the transaction can advance.
    pub fn requires_decision(&self) -> bool {
        matches!(
            self,
            TransactionState::Received
                | TransactionState::PolicyRequired
                | TransactionState::ReadyToSettle
        )
    }
}

impl fmt::Display for TransactionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionState::Received => write!(f, "received"),
            TransactionState::PolicyRequired => write!(f, "policy_required"),
            TransactionState::PartiallyAuthorized => write!(f, "partially_authorized"),
            TransactionState::ReadyToSettle => write!(f, "ready_to_settle"),
            TransactionState::Settled => write!(f, "settled"),
            TransactionState::Rejected => write!(f, "rejected"),
            TransactionState::Cancelled => write!(f, "cancelled"),
            TransactionState::Reverted => write!(f, "reverted"),
        }
    }
}

// ---------------------------------------------------------------------------
// Per-Agent States
// ---------------------------------------------------------------------------

/// Authorization state of an individual agent within a transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    /// Agent has been added to the transaction but has not yet responded.
    Pending,

    /// Agent has sent an Authorize message.
    Authorized,

    /// Agent has sent a Reject message.
    Rejected,

    /// Agent has been removed from the transaction.
    Removed,
}

impl fmt::Display for AgentState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentState::Pending => write!(f, "pending"),
            AgentState::Authorized => write!(f, "authorized"),
            AgentState::Rejected => write!(f, "rejected"),
            AgentState::Removed => write!(f, "removed"),
        }
    }
}

// ---------------------------------------------------------------------------
// Events (incoming messages that drive transitions)
// ---------------------------------------------------------------------------

/// An event that can trigger a state transition in the FSM.
///
/// Each variant corresponds to a TAP message type that affects transaction
/// state. Events carry only the data needed for the state transition, not
/// the full message payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FsmEvent {
    /// A new Transfer or Payment was received, initiating the transaction.
    TransactionReceived {
        /// DIDs of agents involved in this transaction.
        agent_dids: Vec<String>,
    },

    /// An agent sent an Authorize message for this transaction.
    AuthorizeReceived {
        /// DID of the agent that authorized.
        agent_did: String,
        /// Optional settlement address provided by the agent.
        settlement_address: Option<String>,
        /// Optional expiry for this authorization.
        expiry: Option<String>,
    },

    /// An agent sent a Reject message for this transaction.
    RejectReceived {
        /// DID of the agent that rejected.
        agent_did: String,
        /// Optional reason for rejection.
        reason: Option<String>,
    },

    /// A party sent a Cancel message.
    CancelReceived {
        /// DID of the party that cancelled.
        by_did: String,
        /// Optional reason for cancellation.
        reason: Option<String>,
    },

    /// A counterparty sent UpdatePolicies, indicating requirements that
    /// must be fulfilled before they will authorize.
    PoliciesReceived {
        /// DID of the party that sent the policies.
        from_did: String,
    },

    /// A Presentation was received satisfying (some) outstanding policies.
    PresentationReceived {
        /// DID of the party that sent the presentation.
        from_did: String,
    },

    /// The originator sent a Settle message with an on-chain reference.
    SettleReceived {
        /// On-chain settlement identifier (CAIP-220).
        settlement_id: Option<String>,
        /// Actual amount settled.
        amount: Option<String>,
    },

    /// A party sent a Revert message for a settled transaction.
    RevertReceived {
        /// DID of the party requesting revert.
        by_did: String,
        /// Reason for reversal.
        reason: String,
    },

    /// New agents were added to the transaction (TAIP-5).
    AgentsAdded {
        /// DIDs of newly added agents.
        agent_dids: Vec<String>,
    },

    /// An agent was removed from the transaction (TAIP-5).
    AgentRemoved {
        /// DID of the removed agent.
        agent_did: String,
    },
}

impl fmt::Display for FsmEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FsmEvent::TransactionReceived { .. } => write!(f, "TransactionReceived"),
            FsmEvent::AuthorizeReceived { agent_did, .. } => {
                write!(f, "AuthorizeReceived({})", agent_did)
            }
            FsmEvent::RejectReceived { agent_did, .. } => {
                write!(f, "RejectReceived({})", agent_did)
            }
            FsmEvent::CancelReceived { by_did, .. } => write!(f, "CancelReceived({})", by_did),
            FsmEvent::PoliciesReceived { from_did } => {
                write!(f, "PoliciesReceived({})", from_did)
            }
            FsmEvent::PresentationReceived { from_did } => {
                write!(f, "PresentationReceived({})", from_did)
            }
            FsmEvent::SettleReceived { .. } => write!(f, "SettleReceived"),
            FsmEvent::RevertReceived { by_did, .. } => write!(f, "RevertReceived({})", by_did),
            FsmEvent::AgentsAdded { agent_dids } => {
                write!(f, "AgentsAdded({})", agent_dids.join(", "))
            }
            FsmEvent::AgentRemoved { agent_did } => write!(f, "AgentRemoved({})", agent_did),
        }
    }
}

// ---------------------------------------------------------------------------
// Decisions (what the external system must resolve)
// ---------------------------------------------------------------------------

/// A decision that an external system must make before the FSM can advance.
///
/// When the FSM reaches a decision point, it returns one of these variants
/// describing the choices available. The external system (compliance engine,
/// human operator, business rules) must call back with the chosen action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Decision {
    /// A new transaction was received. The external system must decide
    /// how to respond.
    ///
    /// **Available actions:**
    /// - Send `Authorize` to approve
    /// - Send `Reject` to deny
    /// - Send `UpdatePolicies` to request more information
    /// - Send `RequestPresentation` to request credentials
    /// - Do nothing (wait for more context)
    AuthorizationRequired {
        /// The transaction ID requiring a decision.
        transaction_id: String,
        /// DIDs of agents that need to make a decision.
        pending_agents: Vec<String>,
    },

    /// Outstanding policies must be satisfied before authorization can
    /// proceed. The external system must gather the required data and
    /// submit it.
    ///
    /// **Available actions:**
    /// - Send `Presentation` with requested credentials
    /// - Send `ConfirmRelationship` to prove agent-party link
    /// - Send `Reject` if policies cannot be satisfied
    /// - Send `Cancel` to abort
    PolicySatisfactionRequired {
        /// The transaction ID.
        transaction_id: String,
        /// DID of the party that requested policies.
        requested_by: String,
    },

    /// All agents have authorized. The originator must decide whether to
    /// execute settlement on-chain and send a Settle message.
    ///
    /// **Available actions:**
    /// - Execute on-chain settlement and send `Settle`
    /// - Send `Cancel` if settlement should not proceed
    SettlementRequired {
        /// The transaction ID.
        transaction_id: String,
    },
}

impl fmt::Display for Decision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Decision::AuthorizationRequired {
                transaction_id,
                pending_agents,
            } => write!(
                f,
                "AuthorizationRequired(tx={}, agents={})",
                transaction_id,
                pending_agents.join(", ")
            ),
            Decision::PolicySatisfactionRequired {
                transaction_id,
                requested_by,
            } => write!(
                f,
                "PolicySatisfactionRequired(tx={}, by={})",
                transaction_id, requested_by
            ),
            Decision::SettlementRequired { transaction_id } => {
                write!(f, "SettlementRequired(tx={})", transaction_id)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Transition result
// ---------------------------------------------------------------------------

/// The outcome of applying an event to the FSM.
#[derive(Debug, Clone)]
pub struct Transition {
    /// The state before the transition.
    pub from_state: TransactionState,
    /// The state after the transition.
    pub to_state: TransactionState,
    /// The event that triggered this transition.
    pub event: FsmEvent,
    /// If the new state is a decision point, this describes the decision
    /// that must be made by the external system.
    pub decision: Option<Decision>,
}

// ---------------------------------------------------------------------------
// Transaction FSM context
// ---------------------------------------------------------------------------

/// Error returned when an invalid transition is attempted.
#[derive(Debug, Clone)]
pub struct InvalidTransition {
    pub current_state: TransactionState,
    pub event: FsmEvent,
    pub reason: String,
}

impl fmt::Display for InvalidTransition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invalid transition: cannot apply {} in state {} ({})",
            self.event, self.current_state, self.reason
        )
    }
}

impl std::error::Error for InvalidTransition {}

/// The in-memory state of a single transaction tracked by the FSM.
///
/// This struct holds the current state plus per-agent tracking. It is the
/// core data structure manipulated by [`TransactionFsm::apply`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionContext {
    /// Unique transaction identifier (the DIDComm message ID of the
    /// initiating Transfer/Payment).
    pub transaction_id: String,

    /// Current top-level state.
    pub state: TransactionState,

    /// Per-agent authorization state keyed by agent DID.
    pub agents: HashMap<String, AgentState>,

    /// Whether outstanding policies have been received that must be
    /// satisfied before authorization can proceed.
    pub has_pending_policies: bool,
}

impl TransactionContext {
    /// Create a new transaction context in the `Received` state.
    pub fn new(transaction_id: String, agent_dids: Vec<String>) -> Self {
        let agents = agent_dids
            .into_iter()
            .map(|did| (did, AgentState::Pending))
            .collect();

        Self {
            transaction_id,
            state: TransactionState::Received,
            agents,
            has_pending_policies: false,
        }
    }

    /// Returns true if all tracked agents have authorized.
    pub fn all_agents_authorized(&self) -> bool {
        if self.agents.is_empty() {
            return true;
        }
        self.agents
            .values()
            .filter(|s| **s != AgentState::Removed)
            .all(|s| *s == AgentState::Authorized)
    }

    /// Returns the DIDs of agents still in `Pending` state.
    pub fn pending_agents(&self) -> Vec<String> {
        self.agents
            .iter()
            .filter(|(_, s)| **s == AgentState::Pending)
            .map(|(did, _)| did.clone())
            .collect()
    }
}

// ---------------------------------------------------------------------------
// The FSM engine
// ---------------------------------------------------------------------------

/// Pure-logic FSM engine for TAP transactions.
///
/// This struct contains no I/O — it operates only on [`TransactionContext`]
/// and returns [`Transition`] values describing what happened and what
/// decisions are needed. The caller (typically the `StandardTransactionProcessor`
/// or a higher-level orchestrator) is responsible for:
///
/// 1. Persisting state changes
/// 2. Publishing events
/// 3. Presenting decisions to external systems
/// 4. Sending response messages
pub struct TransactionFsm;

impl TransactionFsm {
    /// Apply an event to a transaction context, producing a state transition.
    ///
    /// Returns `Ok(Transition)` on success, describing the state change and
    /// any decision required. Returns `Err(InvalidTransition)` if the event
    /// is not valid in the current state.
    pub fn apply(
        ctx: &mut TransactionContext,
        event: FsmEvent,
    ) -> Result<Transition, InvalidTransition> {
        let from_state = ctx.state.clone();

        // Terminal states accept no further events
        if ctx.state.is_terminal() {
            return Err(InvalidTransition {
                current_state: from_state,
                event,
                reason: "transaction is in a terminal state".to_string(),
            });
        }

        match (&ctx.state, &event) {
            // ----- Initiation -----
            (TransactionState::Received, FsmEvent::TransactionReceived { .. }) => {
                // This is the initial setup — context was just created.
                // Stay in Received; the decision is whether to authorize.
                let decision = Some(Decision::AuthorizationRequired {
                    transaction_id: ctx.transaction_id.clone(),
                    pending_agents: ctx.pending_agents(),
                });
                Ok(Transition {
                    from_state,
                    to_state: ctx.state.clone(),
                    event,
                    decision,
                })
            }

            // ----- Policy exchange -----
            (
                TransactionState::Received | TransactionState::PolicyRequired,
                FsmEvent::PoliciesReceived { from_did },
            ) => {
                ctx.has_pending_policies = true;
                ctx.state = TransactionState::PolicyRequired;
                let decision = Some(Decision::PolicySatisfactionRequired {
                    transaction_id: ctx.transaction_id.clone(),
                    requested_by: from_did.clone(),
                });
                Ok(Transition {
                    from_state,
                    to_state: ctx.state.clone(),
                    event,
                    decision,
                })
            }

            (TransactionState::PolicyRequired, FsmEvent::PresentationReceived { .. }) => {
                // Presentation received — assume policies are satisfied for now.
                // A real implementation would check if ALL policies are met.
                ctx.has_pending_policies = false;
                ctx.state = TransactionState::Received;
                let decision = Some(Decision::AuthorizationRequired {
                    transaction_id: ctx.transaction_id.clone(),
                    pending_agents: ctx.pending_agents(),
                });
                Ok(Transition {
                    from_state,
                    to_state: ctx.state.clone(),
                    event,
                    decision,
                })
            }

            // ----- Authorization -----
            (
                TransactionState::Received
                | TransactionState::PartiallyAuthorized
                | TransactionState::PolicyRequired,
                FsmEvent::AuthorizeReceived { agent_did, .. },
            ) => {
                // Record the agent's authorization
                if let Some(agent_state) = ctx.agents.get_mut(agent_did) {
                    *agent_state = AgentState::Authorized;
                }

                // Determine new transaction state
                if ctx.all_agents_authorized() {
                    ctx.state = TransactionState::ReadyToSettle;
                    let decision = Some(Decision::SettlementRequired {
                        transaction_id: ctx.transaction_id.clone(),
                    });
                    Ok(Transition {
                        from_state,
                        to_state: ctx.state.clone(),
                        event,
                        decision,
                    })
                } else {
                    ctx.state = TransactionState::PartiallyAuthorized;
                    Ok(Transition {
                        from_state,
                        to_state: ctx.state.clone(),
                        event,
                        decision: None,
                    })
                }
            }

            // Authorization can also arrive in ReadyToSettle if a new agent
            // was added after others already authorized.
            (TransactionState::ReadyToSettle, FsmEvent::AuthorizeReceived { agent_did, .. }) => {
                if let Some(agent_state) = ctx.agents.get_mut(agent_did) {
                    *agent_state = AgentState::Authorized;
                }
                // Re-check if all are still authorized
                if ctx.all_agents_authorized() {
                    let decision = Some(Decision::SettlementRequired {
                        transaction_id: ctx.transaction_id.clone(),
                    });
                    Ok(Transition {
                        from_state,
                        to_state: ctx.state.clone(),
                        event,
                        decision,
                    })
                } else {
                    ctx.state = TransactionState::PartiallyAuthorized;
                    Ok(Transition {
                        from_state,
                        to_state: ctx.state.clone(),
                        event,
                        decision: None,
                    })
                }
            }

            // ----- Rejection -----
            (_, FsmEvent::RejectReceived { agent_did, .. }) => {
                if let Some(agent_state) = ctx.agents.get_mut(agent_did) {
                    *agent_state = AgentState::Rejected;
                }
                ctx.state = TransactionState::Rejected;
                Ok(Transition {
                    from_state,
                    to_state: ctx.state.clone(),
                    event,
                    decision: None,
                })
            }

            // ----- Cancellation -----
            (_, FsmEvent::CancelReceived { .. }) => {
                ctx.state = TransactionState::Cancelled;
                Ok(Transition {
                    from_state,
                    to_state: ctx.state.clone(),
                    event,
                    decision: None,
                })
            }

            // ----- Settlement -----
            (TransactionState::ReadyToSettle, FsmEvent::SettleReceived { .. }) => {
                ctx.state = TransactionState::Settled;
                Ok(Transition {
                    from_state,
                    to_state: ctx.state.clone(),
                    event,
                    decision: None,
                })
            }

            // ----- Revert -----
            (TransactionState::Settled, FsmEvent::RevertReceived { .. }) => {
                ctx.state = TransactionState::Reverted;
                Ok(Transition {
                    from_state,
                    to_state: ctx.state.clone(),
                    event,
                    decision: None,
                })
            }

            // ----- Agent management (TAIP-5) -----
            (_, FsmEvent::AgentsAdded { agent_dids }) => {
                for did in agent_dids {
                    ctx.agents.entry(did.clone()).or_insert(AgentState::Pending);
                }
                // Adding agents may move us out of ReadyToSettle if new
                // agents are pending.
                if from_state == TransactionState::ReadyToSettle && !ctx.all_agents_authorized() {
                    ctx.state = TransactionState::PartiallyAuthorized;
                }
                Ok(Transition {
                    from_state,
                    to_state: ctx.state.clone(),
                    event,
                    decision: None,
                })
            }

            (_, FsmEvent::AgentRemoved { agent_did }) => {
                if let Some(agent_state) = ctx.agents.get_mut(agent_did) {
                    *agent_state = AgentState::Removed;
                }
                // Removing an agent may make all remaining agents authorized.
                if matches!(
                    ctx.state,
                    TransactionState::PartiallyAuthorized | TransactionState::Received
                ) && ctx.all_agents_authorized()
                {
                    ctx.state = TransactionState::ReadyToSettle;
                    let decision = Some(Decision::SettlementRequired {
                        transaction_id: ctx.transaction_id.clone(),
                    });
                    return Ok(Transition {
                        from_state,
                        to_state: ctx.state.clone(),
                        event,
                        decision,
                    });
                }
                Ok(Transition {
                    from_state,
                    to_state: ctx.state.clone(),
                    event,
                    decision: None,
                })
            }

            // ----- Invalid transitions -----
            _ => Err(InvalidTransition {
                current_state: from_state,
                event: event.clone(),
                reason: format!("event {} is not valid in state {}", event, ctx.state),
            }),
        }
    }

    /// Returns all valid events for a given state (for documentation/UI).
    pub fn valid_events(state: &TransactionState) -> Vec<&'static str> {
        match state {
            TransactionState::Received => vec![
                "TransactionReceived",
                "AuthorizeReceived",
                "RejectReceived",
                "CancelReceived",
                "PoliciesReceived",
                "AgentsAdded",
                "AgentRemoved",
            ],
            TransactionState::PolicyRequired => vec![
                "PresentationReceived",
                "PoliciesReceived",
                "AuthorizeReceived",
                "RejectReceived",
                "CancelReceived",
                "AgentsAdded",
                "AgentRemoved",
            ],
            TransactionState::PartiallyAuthorized => vec![
                "AuthorizeReceived",
                "RejectReceived",
                "CancelReceived",
                "AgentsAdded",
                "AgentRemoved",
            ],
            TransactionState::ReadyToSettle => vec![
                "SettleReceived",
                "AuthorizeReceived",
                "RejectReceived",
                "CancelReceived",
                "AgentsAdded",
                "AgentRemoved",
            ],
            TransactionState::Settled => vec!["RevertReceived"],
            TransactionState::Rejected
            | TransactionState::Cancelled
            | TransactionState::Reverted => {
                vec![]
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Decision handler configuration
// ---------------------------------------------------------------------------

/// Controls how the node handles decision points during transaction
/// processing.
///
/// This enum is set on [`NodeConfig`] and determines which
/// [`DecisionHandler`] implementation the `StandardTransactionProcessor`
/// uses at runtime.
#[derive(Debug, Clone, Default)]
pub enum DecisionMode {
    /// Automatically approve all decisions — the node will immediately
    /// send Authorize messages for registered agents and Settle when all
    /// agents have authorized. This is the current default behavior and
    /// is suitable for testing or fully-automated deployments.
    #[default]
    AutoApprove,

    /// Publish each decision as a [`NodeEvent::DecisionRequired`] on the
    /// event bus. No automatic action is taken — an external subscriber
    /// (compliance engine, human operator UI, business rules engine) must
    /// listen for these events and call back into the node to advance the
    /// transaction.
    EventBus,

    /// Use a custom decision handler provided by the caller.
    Custom(Arc<dyn DecisionHandler>),
}

// ---------------------------------------------------------------------------
// Decision handler trait
// ---------------------------------------------------------------------------

/// Trait for handling FSM decision points.
///
/// When the FSM reaches a state that requires an external decision
/// (e.g., whether to authorize a new transfer), the
/// `StandardTransactionProcessor` calls the configured `DecisionHandler`.
///
/// Implementations can auto-approve, publish to an event bus, call out
/// to a compliance API, present a UI to a human operator, etc.
#[async_trait]
pub trait DecisionHandler: Send + Sync + fmt::Debug {
    /// Called when the FSM produces a [`Decision`].
    ///
    /// The handler receives the full [`TransactionContext`] (current state,
    /// per-agent status) and the [`Decision`] describing what needs to be
    /// resolved.
    ///
    /// Implementations that auto-resolve should return the same `Decision`
    /// back. Implementations that defer to external systems should publish
    /// the decision and return it for auditing.
    async fn handle_decision(&self, ctx: &TransactionContext, decision: &Decision);
}

// ---------------------------------------------------------------------------
// Built-in: AutoApproveHandler
// ---------------------------------------------------------------------------

/// Decision handler that automatically approves all decisions.
///
/// - `AuthorizationRequired` → queues Authorize messages for all registered
///   agents (the actual sending is done by the processor, not this handler)
/// - `SettlementRequired` → allows the processor to send Settle
/// - `PolicySatisfactionRequired` → logged, no action (policies are not
///   auto-satisfiable)
///
/// This preserves the existing tap-node behavior where registered agents
/// are auto-authorized and settlement is automatic.
#[derive(Debug)]
pub struct AutoApproveHandler;

#[async_trait]
impl DecisionHandler for AutoApproveHandler {
    async fn handle_decision(&self, _ctx: &TransactionContext, decision: &Decision) {
        log::debug!("AutoApproveHandler: auto-resolving {}", decision);
    }
}

// ---------------------------------------------------------------------------
// Built-in: LogOnlyHandler
// ---------------------------------------------------------------------------

/// Decision handler that only logs decisions without taking action.
///
/// Useful for monitoring/observability when an external system handles
/// decisions through the event bus channel subscription instead of the
/// `DecisionHandler` trait.
#[derive(Debug)]
pub struct LogOnlyHandler;

#[async_trait]
impl DecisionHandler for LogOnlyHandler {
    async fn handle_decision(&self, ctx: &TransactionContext, decision: &Decision) {
        log::info!(
            "Decision required for transaction {} (state={}): {}",
            ctx.transaction_id,
            ctx.state,
            decision
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx(agents: &[&str]) -> TransactionContext {
        TransactionContext::new(
            "tx-001".to_string(),
            agents.iter().map(|s| s.to_string()).collect(),
        )
    }

    #[test]
    fn test_happy_path_single_agent() {
        let mut ctx = make_ctx(&["did:example:compliance"]);
        assert_eq!(ctx.state, TransactionState::Received);

        // Receive transaction
        let t = TransactionFsm::apply(
            &mut ctx,
            FsmEvent::TransactionReceived {
                agent_dids: vec!["did:example:compliance".to_string()],
            },
        )
        .unwrap();
        assert_eq!(t.to_state, TransactionState::Received);
        assert!(t.decision.is_some());
        assert!(matches!(
            t.decision.unwrap(),
            Decision::AuthorizationRequired { .. }
        ));

        // Agent authorizes
        let t = TransactionFsm::apply(
            &mut ctx,
            FsmEvent::AuthorizeReceived {
                agent_did: "did:example:compliance".to_string(),
                settlement_address: None,
                expiry: None,
            },
        )
        .unwrap();
        assert_eq!(t.to_state, TransactionState::ReadyToSettle);
        assert!(matches!(
            t.decision.unwrap(),
            Decision::SettlementRequired { .. }
        ));

        // Settle
        let t = TransactionFsm::apply(
            &mut ctx,
            FsmEvent::SettleReceived {
                settlement_id: Some("eip155:1:tx/0xabc".to_string()),
                amount: None,
            },
        )
        .unwrap();
        assert_eq!(t.to_state, TransactionState::Settled);
        assert!(t.decision.is_none());
    }

    #[test]
    fn test_happy_path_multi_agent() {
        let mut ctx = make_ctx(&["did:example:a", "did:example:b"]);

        // First agent authorizes
        let t = TransactionFsm::apply(
            &mut ctx,
            FsmEvent::AuthorizeReceived {
                agent_did: "did:example:a".to_string(),
                settlement_address: None,
                expiry: None,
            },
        )
        .unwrap();
        assert_eq!(t.to_state, TransactionState::PartiallyAuthorized);
        assert!(t.decision.is_none());

        // Second agent authorizes
        let t = TransactionFsm::apply(
            &mut ctx,
            FsmEvent::AuthorizeReceived {
                agent_did: "did:example:b".to_string(),
                settlement_address: None,
                expiry: None,
            },
        )
        .unwrap();
        assert_eq!(t.to_state, TransactionState::ReadyToSettle);
        assert!(matches!(
            t.decision.unwrap(),
            Decision::SettlementRequired { .. }
        ));
    }

    #[test]
    fn test_rejection() {
        let mut ctx = make_ctx(&["did:example:a"]);

        let t = TransactionFsm::apply(
            &mut ctx,
            FsmEvent::RejectReceived {
                agent_did: "did:example:a".to_string(),
                reason: Some("sanctions screening failed".to_string()),
            },
        )
        .unwrap();
        assert_eq!(t.to_state, TransactionState::Rejected);
        assert!(ctx.state.is_terminal());
    }

    #[test]
    fn test_cancellation() {
        let mut ctx = make_ctx(&["did:example:a"]);

        let t = TransactionFsm::apply(
            &mut ctx,
            FsmEvent::CancelReceived {
                by_did: "did:example:originator".to_string(),
                reason: None,
            },
        )
        .unwrap();
        assert_eq!(t.to_state, TransactionState::Cancelled);
        assert!(ctx.state.is_terminal());
    }

    #[test]
    fn test_policy_flow() {
        let mut ctx = make_ctx(&["did:example:a"]);

        // Counterparty sends policies
        let t = TransactionFsm::apply(
            &mut ctx,
            FsmEvent::PoliciesReceived {
                from_did: "did:example:beneficiary-vasp".to_string(),
            },
        )
        .unwrap();
        assert_eq!(t.to_state, TransactionState::PolicyRequired);
        assert!(matches!(
            t.decision.unwrap(),
            Decision::PolicySatisfactionRequired { .. }
        ));

        // We send a presentation satisfying the policies
        let t = TransactionFsm::apply(
            &mut ctx,
            FsmEvent::PresentationReceived {
                from_did: "did:example:originator-vasp".to_string(),
            },
        )
        .unwrap();
        assert_eq!(t.to_state, TransactionState::Received);
        assert!(matches!(
            t.decision.unwrap(),
            Decision::AuthorizationRequired { .. }
        ));

        // Now agent can authorize
        let t = TransactionFsm::apply(
            &mut ctx,
            FsmEvent::AuthorizeReceived {
                agent_did: "did:example:a".to_string(),
                settlement_address: None,
                expiry: None,
            },
        )
        .unwrap();
        assert_eq!(t.to_state, TransactionState::ReadyToSettle);
    }

    #[test]
    fn test_revert() {
        let mut ctx = make_ctx(&["did:example:a"]);

        // Authorize → Settle → Revert
        TransactionFsm::apply(
            &mut ctx,
            FsmEvent::AuthorizeReceived {
                agent_did: "did:example:a".to_string(),
                settlement_address: None,
                expiry: None,
            },
        )
        .unwrap();

        TransactionFsm::apply(
            &mut ctx,
            FsmEvent::SettleReceived {
                settlement_id: None,
                amount: None,
            },
        )
        .unwrap();

        let t = TransactionFsm::apply(
            &mut ctx,
            FsmEvent::RevertReceived {
                by_did: "did:example:beneficiary".to_string(),
                reason: "incorrect amount".to_string(),
            },
        )
        .unwrap();
        assert_eq!(t.to_state, TransactionState::Reverted);
        assert!(ctx.state.is_terminal());
    }

    #[test]
    fn test_terminal_state_rejects_events() {
        let mut ctx = make_ctx(&["did:example:a"]);
        ctx.state = TransactionState::Rejected;

        let result = TransactionFsm::apply(
            &mut ctx,
            FsmEvent::AuthorizeReceived {
                agent_did: "did:example:a".to_string(),
                settlement_address: None,
                expiry: None,
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_settle_only_from_ready_to_settle() {
        let mut ctx = make_ctx(&["did:example:a"]);
        // Still in Received state, try to settle
        let result = TransactionFsm::apply(
            &mut ctx,
            FsmEvent::SettleReceived {
                settlement_id: None,
                amount: None,
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_add_agents_blocks_settlement() {
        let mut ctx = make_ctx(&["did:example:a"]);

        // Authorize the first agent
        TransactionFsm::apply(
            &mut ctx,
            FsmEvent::AuthorizeReceived {
                agent_did: "did:example:a".to_string(),
                settlement_address: None,
                expiry: None,
            },
        )
        .unwrap();
        assert_eq!(ctx.state, TransactionState::ReadyToSettle);

        // Add a new agent — should move back to PartiallyAuthorized
        let t = TransactionFsm::apply(
            &mut ctx,
            FsmEvent::AgentsAdded {
                agent_dids: vec!["did:example:b".to_string()],
            },
        )
        .unwrap();
        assert_eq!(t.to_state, TransactionState::PartiallyAuthorized);
    }

    #[test]
    fn test_remove_agent_enables_settlement() {
        let mut ctx = make_ctx(&["did:example:a", "did:example:b"]);

        // Only authorize agent a
        TransactionFsm::apply(
            &mut ctx,
            FsmEvent::AuthorizeReceived {
                agent_did: "did:example:a".to_string(),
                settlement_address: None,
                expiry: None,
            },
        )
        .unwrap();
        assert_eq!(ctx.state, TransactionState::PartiallyAuthorized);

        // Remove agent b — now all remaining agents are authorized
        let t = TransactionFsm::apply(
            &mut ctx,
            FsmEvent::AgentRemoved {
                agent_did: "did:example:b".to_string(),
            },
        )
        .unwrap();
        assert_eq!(t.to_state, TransactionState::ReadyToSettle);
        assert!(matches!(
            t.decision.unwrap(),
            Decision::SettlementRequired { .. }
        ));
    }

    #[test]
    fn test_no_agents_goes_straight_to_ready() {
        let mut ctx = make_ctx(&[]);

        // With no agents, any authorize immediately reaches ReadyToSettle.
        // Actually, with no agents all_agents_authorized() is true from the
        // start. A TransactionReceived should reflect this.
        let t = TransactionFsm::apply(
            &mut ctx,
            FsmEvent::TransactionReceived { agent_dids: vec![] },
        )
        .unwrap();
        // Still Received — the decision is AuthorizationRequired even with
        // no agents, because the *parties* themselves may still need to decide.
        assert_eq!(t.to_state, TransactionState::Received);
    }

    #[test]
    fn test_valid_events() {
        let events = TransactionFsm::valid_events(&TransactionState::Received);
        assert!(events.contains(&"AuthorizeReceived"));
        assert!(events.contains(&"PoliciesReceived"));

        let events = TransactionFsm::valid_events(&TransactionState::Settled);
        assert_eq!(events, vec!["RevertReceived"]);

        let events = TransactionFsm::valid_events(&TransactionState::Rejected);
        assert!(events.is_empty());
    }

    #[test]
    fn test_display_implementations() {
        assert_eq!(TransactionState::Received.to_string(), "received");
        assert_eq!(
            TransactionState::PolicyRequired.to_string(),
            "policy_required"
        );
        assert_eq!(
            TransactionState::PartiallyAuthorized.to_string(),
            "partially_authorized"
        );
        assert_eq!(
            TransactionState::ReadyToSettle.to_string(),
            "ready_to_settle"
        );
        assert_eq!(TransactionState::Settled.to_string(), "settled");
        assert_eq!(TransactionState::Rejected.to_string(), "rejected");
        assert_eq!(TransactionState::Cancelled.to_string(), "cancelled");
        assert_eq!(TransactionState::Reverted.to_string(), "reverted");
    }

    #[test]
    fn test_agent_state_display() {
        assert_eq!(AgentState::Pending.to_string(), "pending");
        assert_eq!(AgentState::Authorized.to_string(), "authorized");
        assert_eq!(AgentState::Rejected.to_string(), "rejected");
        assert_eq!(AgentState::Removed.to_string(), "removed");
    }

    #[test]
    fn test_reject_during_partial_authorization() {
        let mut ctx = make_ctx(&["did:example:a", "did:example:b"]);

        // Agent a authorizes
        TransactionFsm::apply(
            &mut ctx,
            FsmEvent::AuthorizeReceived {
                agent_did: "did:example:a".to_string(),
                settlement_address: None,
                expiry: None,
            },
        )
        .unwrap();
        assert_eq!(ctx.state, TransactionState::PartiallyAuthorized);

        // Agent b rejects
        let t = TransactionFsm::apply(
            &mut ctx,
            FsmEvent::RejectReceived {
                agent_did: "did:example:b".to_string(),
                reason: Some("compliance failure".to_string()),
            },
        )
        .unwrap();
        assert_eq!(t.to_state, TransactionState::Rejected);
    }

    #[test]
    fn test_cancel_during_ready_to_settle() {
        let mut ctx = make_ctx(&["did:example:a"]);

        TransactionFsm::apply(
            &mut ctx,
            FsmEvent::AuthorizeReceived {
                agent_did: "did:example:a".to_string(),
                settlement_address: None,
                expiry: None,
            },
        )
        .unwrap();
        assert_eq!(ctx.state, TransactionState::ReadyToSettle);

        // Cancel even though ready to settle
        let t = TransactionFsm::apply(
            &mut ctx,
            FsmEvent::CancelReceived {
                by_did: "did:example:originator".to_string(),
                reason: Some("changed mind".to_string()),
            },
        )
        .unwrap();
        assert_eq!(t.to_state, TransactionState::Cancelled);
    }

    #[test]
    fn test_authorize_from_policy_required() {
        let mut ctx = make_ctx(&["did:example:a"]);

        // Receive policies
        TransactionFsm::apply(
            &mut ctx,
            FsmEvent::PoliciesReceived {
                from_did: "did:example:b".to_string(),
            },
        )
        .unwrap();
        assert_eq!(ctx.state, TransactionState::PolicyRequired);

        // Agent can still authorize even in PolicyRequired state
        // (they may have already satisfied the policies externally)
        let t = TransactionFsm::apply(
            &mut ctx,
            FsmEvent::AuthorizeReceived {
                agent_did: "did:example:a".to_string(),
                settlement_address: None,
                expiry: None,
            },
        )
        .unwrap();
        assert_eq!(t.to_state, TransactionState::ReadyToSettle);
    }
}
