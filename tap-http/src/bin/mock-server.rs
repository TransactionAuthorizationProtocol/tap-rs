use http_body_util::BodyExt;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// Handle requests and return appropriate responses
async fn handle_request(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<String>, Infallible> {
    println!("Received request: {:?}", req);

    // Handle different routes
    match (req.method(), req.uri().path()) {
        // Health check endpoint
        (&Method::GET, "/health") => {
            println!("Health check requested");
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(r#"{"status":"ok","version":"0.1.0"}"#.to_string())
                .unwrap();
            Ok(response)
        }

        // DIDComm messages endpoint
        (&Method::POST, "/didcomm") => {
            println!("DIDComm message received");

            // Get the request body
            let body_bytes = req.into_body().collect().await.unwrap().to_bytes();
            let body_str = String::from_utf8_lossy(&body_bytes);

            // Print the request content
            println!("Received message body:");
            println!("{}", body_str);

            // Try to parse as JSON to verify structure
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body_str) {
                // Check if the message has signature information
                if let Some(signatures) = json.get("signatures") {
                    println!("\nSignature information detected:");
                    println!("{}", serde_json::to_string_pretty(signatures).unwrap());
                }

                // Check from/to fields
                if let Some(from) = json.get("from") {
                    println!("\nFrom: {}", from);
                }

                if let Some(to) = json.get("to") {
                    println!("\nTo: {}", to);
                }
            }

            // Return success response
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(r#"{"status":"success","message":"Message received and logged"}"#.to_string())
                .unwrap();
            Ok(response)
        }

        // Fallback for all other requests
        _ => {
            println!("Unknown route requested: {}", req.uri().path());
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("Not Found".to_string())
                .unwrap();
            Ok(response)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Configure the server address
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Starting mock TAP server on {}", addr);

    // Create a TCP listener
    let listener = TcpListener::bind(addr).await?;
    println!("Server started. Press Ctrl+C to stop.");

    // Accept connections in a loop
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        // Spawn a task to handle the connection
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handle_request))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
