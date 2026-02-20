use serde::Serialize;
use serde_json::Value;

/// Output format for CLI responses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Text,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "text" => Ok(Self::Text),
            _ => Err(format!("Unknown format: {}. Use 'json' or 'text'", s)),
        }
    }
}

/// Wrapper for consistent CLI output
#[derive(Debug, Serialize)]
struct SuccessEnvelope<T: Serialize> {
    status: &'static str,
    data: T,
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope {
    status: &'static str,
    error: String,
}

/// Print a successful result in the chosen format
pub fn print_success<T: Serialize>(format: OutputFormat, data: &T) {
    match format {
        OutputFormat::Json => {
            let envelope = SuccessEnvelope {
                status: "success",
                data,
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&envelope).unwrap_or_else(|e| format!(
                    "{{\"status\":\"error\",\"error\":\"Serialization failed: {}\"}}",
                    e
                ))
            );
        }
        OutputFormat::Text => {
            // For text mode, pretty-print the data as JSON for now
            // Individual commands can override with custom formatting
            let json = serde_json::to_value(data).unwrap_or(Value::Null);
            print_text_value(&json, 0);
        }
    }
}

/// Print an error in the chosen format
pub fn print_error(format: OutputFormat, error: &str) {
    match format {
        OutputFormat::Json => {
            let envelope = ErrorEnvelope {
                status: "error",
                error: error.to_string(),
            };
            eprintln!(
                "{}",
                serde_json::to_string_pretty(&envelope).unwrap_or_else(|e| format!(
                    "{{\"status\":\"error\",\"error\":\"Serialization failed: {}\"}}",
                    e
                ))
            );
        }
        OutputFormat::Text => {
            eprintln!("Error: {}", error);
        }
    }
}

/// Recursively print a JSON value in human-readable text format
fn print_text_value(value: &Value, indent: usize) {
    let pad = " ".repeat(indent);
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                match val {
                    Value::Object(_) | Value::Array(_) => {
                        println!("{}{}:", pad, key);
                        print_text_value(val, indent + 2);
                    }
                    _ => {
                        println!("{}{}: {}", pad, key, format_scalar(val));
                    }
                }
            }
        }
        Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                println!("{}[{}]:", pad, i);
                print_text_value(val, indent + 2);
            }
        }
        _ => {
            println!("{}{}", pad, format_scalar(value));
        }
    }
}

fn format_scalar(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        _ => value.to_string(),
    }
}
