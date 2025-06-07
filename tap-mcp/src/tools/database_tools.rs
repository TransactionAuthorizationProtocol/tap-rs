//! Database query tools for read-only SQL access

use crate::error::{Error, Result};
use crate::mcp::protocol::{CallToolResult, Tool};
use crate::tap_integration::TapIntegration;
use crate::tools::{error_text_response, success_text_response, ToolHandler};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{Connection, Row, SqliteConnection};
use std::sync::Arc;
use tracing::{debug, error};

/// Tool for executing read-only SQL queries
pub struct QueryDatabaseTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for querying the database
#[derive(Debug, Deserialize)]
struct QueryDatabaseParams {
    agent_did: String,
    query: String,
}

/// Response for database query
#[derive(Debug, Serialize)]
struct QueryDatabaseResponse {
    columns: Vec<String>,
    rows: Vec<Vec<Value>>,
    row_count: usize,
    query: String,
}

impl QueryDatabaseTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }

    /// Check if a query is read-only
    fn is_read_only_query(query: &str) -> bool {
        let query_upper = query.trim().to_uppercase();
        let forbidden_keywords = [
            "INSERT",
            "UPDATE",
            "DELETE",
            "DROP",
            "CREATE",
            "ALTER",
            "TRUNCATE",
            "REPLACE",
            "MERGE",
            "CALL",
            "EXECUTE",
            "EXEC",
            "BEGIN",
            "COMMIT",
            "ROLLBACK",
            "SAVEPOINT",
            "GRANT",
            "REVOKE",
            "DENY",
            "ATTACH",
            "DETACH",
        ];

        // Check if query starts with SELECT, WITH, or PRAGMA (for schema queries)
        let allowed_starts = ["SELECT", "WITH", "PRAGMA", "EXPLAIN"];
        let starts_with_allowed = allowed_starts
            .iter()
            .any(|&start| query_upper.starts_with(start));

        // Check for forbidden keywords
        let contains_forbidden = forbidden_keywords.iter().any(|&keyword| {
            // Check for whole word matches to avoid false positives
            query_upper.split_whitespace().any(|word| word == keyword)
        });

        starts_with_allowed && !contains_forbidden
    }
}

#[async_trait]
impl ToolHandler for QueryDatabaseTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: QueryDatabaseParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!(
            "Executing query for agent {}: {}",
            params.agent_did, params.query
        );

        // Validate query is read-only
        if !Self::is_read_only_query(&params.query) {
            return Ok(error_text_response(
                "Only read-only queries are allowed. Query must start with SELECT, WITH, PRAGMA, or EXPLAIN and cannot contain modification keywords.".to_string(),
            ));
        }

        // Get agent storage
        let storage = match self
            .tap_integration
            .storage_for_agent(&params.agent_did)
            .await
        {
            Ok(storage) => storage,
            Err(e) => {
                error!(
                    "Failed to get storage for agent {}: {}",
                    params.agent_did, e
                );
                return Ok(error_text_response(format!(
                    "Failed to get storage for agent {}: {}",
                    params.agent_did, e
                )));
            }
        };

        // Get database path from storage
        let db_path = storage.db_path();
        let db_url = format!("sqlite://{}?mode=ro", db_path.display());

        // Connect to database in read-only mode
        let mut conn = match SqliteConnection::connect(&db_url).await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to connect to database: {}", e);
                return Ok(error_text_response(format!(
                    "Failed to connect to database: {}",
                    e
                )));
            }
        };

        // Execute query
        match sqlx::query(&params.query).fetch_all(&mut conn).await {
            Ok(rows) => {
                let mut columns = Vec::new();
                let mut result_rows = Vec::new();

                if !rows.is_empty() {
                    // Get column names from the first row
                    let first_row = &rows[0];
                    for (i, column) in first_row.columns().iter().enumerate() {
                        columns.push(column.name().to_string());
                    }

                    // Process all rows
                    for row in &rows {
                        let mut row_values = Vec::new();
                        for i in 0..columns.len() {
                            // Try different types in order of likelihood
                            let value = if let Ok(v) = row.try_get::<Option<i64>, _>(i) {
                                v.map(Value::from).unwrap_or(Value::Null)
                            } else if let Ok(v) = row.try_get::<Option<f64>, _>(i) {
                                v.map(Value::from).unwrap_or(Value::Null)
                            } else if let Ok(v) = row.try_get::<Option<String>, _>(i) {
                                v.map(Value::from).unwrap_or(Value::Null)
                            } else if let Ok(v) = row.try_get::<Option<bool>, _>(i) {
                                v.map(Value::from).unwrap_or(Value::Null)
                            } else if let Ok(v) = row.try_get::<Option<Vec<u8>>, _>(i) {
                                v.map(|bytes| {
                                    // Try to convert bytes to string if possible
                                    if let Ok(s) = String::from_utf8(bytes.clone()) {
                                        Value::String(s)
                                    } else {
                                        // Return as base64 encoded string
                                        use base64::Engine;
                                        Value::String(
                                            base64::engine::general_purpose::STANDARD.encode(bytes),
                                        )
                                    }
                                })
                                .unwrap_or(Value::Null)
                            } else {
                                Value::Null
                            };
                            row_values.push(value);
                        }
                        result_rows.push(row_values);
                    }
                }

                let response = QueryDatabaseResponse {
                    columns,
                    row_count: result_rows.len(),
                    rows: result_rows,
                    query: params.query,
                };

                let response_json = serde_json::to_string_pretty(&response).map_err(|e| {
                    Error::tool_execution(format!("Failed to serialize response: {}", e))
                })?;

                Ok(success_text_response(response_json))
            }
            Err(e) => {
                error!("Failed to execute query: {}", e);
                Ok(error_text_response(format!(
                    "Failed to execute query: {}",
                    e
                )))
            }
        }
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_query_database".to_string(),
            description: "Executes read-only SQL queries on an agent's database. Only SELECT, WITH, PRAGMA, and EXPLAIN queries are allowed.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "agent_did": {
                        "type": "string",
                        "description": "The DID of the agent whose database to query"
                    },
                    "query": {
                        "type": "string",
                        "description": "The read-only SQL query to execute"
                    }
                },
                "required": ["agent_did", "query"],
                "additionalProperties": false
            }),
        }
    }
}

/// Tool for getting database schema
pub struct GetDatabaseSchemaTool {
    tap_integration: Arc<TapIntegration>,
}

/// Parameters for getting database schema
#[derive(Debug, Deserialize)]
struct GetDatabaseSchemaParams {
    agent_did: String,
    #[serde(default)]
    table_name: Option<String>,
}

/// Table information
#[derive(Debug, Serialize)]
struct TableInfo {
    name: String,
    columns: Vec<ColumnInfo>,
    indexes: Vec<IndexInfo>,
    row_count: i64,
}

/// Column information
#[derive(Debug, Serialize)]
struct ColumnInfo {
    cid: i32,
    name: String,
    #[serde(rename = "type")]
    column_type: String,
    notnull: bool,
    dflt_value: Option<String>,
    pk: bool,
}

/// Index information
#[derive(Debug, Serialize)]
struct IndexInfo {
    name: String,
    unique: bool,
    origin: String,
    partial: bool,
}

/// Response for database schema
#[derive(Debug, Serialize)]
struct GetDatabaseSchemaResponse {
    database_path: String,
    tables: Vec<TableInfo>,
}

impl GetDatabaseSchemaTool {
    pub fn new(tap_integration: Arc<TapIntegration>) -> Self {
        Self { tap_integration }
    }
}

#[async_trait]
impl ToolHandler for GetDatabaseSchemaTool {
    async fn handle(&self, arguments: Option<Value>) -> Result<CallToolResult> {
        let params: GetDatabaseSchemaParams = match arguments {
            Some(args) => serde_json::from_value(args)
                .map_err(|e| Error::invalid_parameter(format!("Invalid parameters: {}", e)))?,
            None => {
                return Ok(error_text_response(
                    "Missing required parameters".to_string(),
                ))
            }
        };

        debug!("Getting database schema for agent {}", params.agent_did);

        // Get agent storage
        let storage = match self
            .tap_integration
            .storage_for_agent(&params.agent_did)
            .await
        {
            Ok(storage) => storage,
            Err(e) => {
                error!(
                    "Failed to get storage for agent {}: {}",
                    params.agent_did, e
                );
                return Ok(error_text_response(format!(
                    "Failed to get storage for agent {}: {}",
                    params.agent_did, e
                )));
            }
        };

        // Get database path from storage
        let db_path = storage.db_path();
        let db_url = format!("sqlite://{}?mode=ro", db_path.display());

        // Connect to database in read-only mode
        let mut conn = match SqliteConnection::connect(&db_url).await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to connect to database: {}", e);
                return Ok(error_text_response(format!(
                    "Failed to connect to database: {}",
                    e
                )));
            }
        };

        let mut tables = Vec::new();

        // Get list of tables
        let table_query = if let Some(ref table_name) = params.table_name {
            format!(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='{}' ORDER BY name",
                table_name
            )
        } else {
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name".to_string()
        };

        let table_rows = match sqlx::query(&table_query).fetch_all(&mut conn).await {
            Ok(rows) => rows,
            Err(e) => {
                error!("Failed to get tables: {}", e);
                return Ok(error_text_response(format!("Failed to get tables: {}", e)));
            }
        };

        for table_row in table_rows {
            let table_name: String = table_row.try_get("name").unwrap_or_default();

            // Get columns for this table
            let column_query = format!("PRAGMA table_info('{}')", table_name);
            let column_rows = match sqlx::query(&column_query).fetch_all(&mut conn).await {
                Ok(rows) => rows,
                Err(e) => {
                    error!("Failed to get columns for table {}: {}", table_name, e);
                    continue;
                }
            };

            let mut columns = Vec::new();
            for col_row in column_rows {
                columns.push(ColumnInfo {
                    cid: col_row.try_get("cid").unwrap_or(0),
                    name: col_row.try_get("name").unwrap_or_default(),
                    column_type: col_row.try_get("type").unwrap_or_default(),
                    notnull: col_row.try_get::<i32, _>("notnull").unwrap_or(0) != 0,
                    dflt_value: col_row.try_get("dflt_value").ok(),
                    pk: col_row.try_get::<i32, _>("pk").unwrap_or(0) != 0,
                });
            }

            // Get indexes for this table
            let index_query = format!("PRAGMA index_list('{}')", table_name);
            let index_rows = match sqlx::query(&index_query).fetch_all(&mut conn).await {
                Ok(rows) => rows,
                Err(e) => {
                    error!("Failed to get indexes for table {}: {}", table_name, e);
                    vec![]
                }
            };

            let mut indexes = Vec::new();
            for idx_row in index_rows {
                indexes.push(IndexInfo {
                    name: idx_row.try_get("name").unwrap_or_default(),
                    unique: idx_row.try_get::<i32, _>("unique").unwrap_or(0) != 0,
                    origin: idx_row.try_get("origin").unwrap_or_default(),
                    partial: idx_row.try_get::<i32, _>("partial").unwrap_or(0) != 0,
                });
            }

            // Get row count
            let count_query = format!("SELECT COUNT(*) as count FROM '{}'", table_name);
            let row_count = match sqlx::query(&count_query).fetch_one(&mut conn).await {
                Ok(row) => row.try_get::<i64, _>("count").unwrap_or(0),
                Err(e) => {
                    error!("Failed to get row count for table {}: {}", table_name, e);
                    0
                }
            };

            tables.push(TableInfo {
                name: table_name,
                columns,
                indexes,
                row_count,
            });
        }

        let response = GetDatabaseSchemaResponse {
            database_path: db_path.display().to_string(),
            tables,
        };

        let response_json = serde_json::to_string_pretty(&response)
            .map_err(|e| Error::tool_execution(format!("Failed to serialize response: {}", e)))?;

        Ok(success_text_response(response_json))
    }

    fn get_definition(&self) -> Tool {
        Tool {
            name: "tap_get_database_schema".to_string(),
            description: "Gets the schema of an agent's database, including all tables, columns, indexes, and row counts. Optionally filter by table name.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "agent_did": {
                        "type": "string",
                        "description": "The DID of the agent whose database schema to retrieve"
                    },
                    "table_name": {
                        "type": "string",
                        "description": "Optional specific table name to get schema for"
                    }
                },
                "required": ["agent_did"],
                "additionalProperties": false
            }),
        }
    }
}
