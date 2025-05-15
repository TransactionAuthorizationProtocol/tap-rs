use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};

/// Handle requests and return appropriate responses
async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    println!("Received request: {:?}", req);
    
    // Handle different routes
    match (req.method(), req.uri().path()) {
        // Health check endpoint
        (&Method::GET, "/health") => {
            println!("Health check requested");
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"status":"ok","version":"0.1.0"}"#))
                .unwrap();
            Ok(response)
        },
        
        // DIDComm messages endpoint
        (&Method::POST, "/didcomm") => {
            println!("DIDComm message received");
            
            // Get the request body
            let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
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
                .body(Body::from(r#"{"status":"success","message":"Message received and logged"}"#))
                .unwrap();
            Ok(response)
        },
        
        // Fallback for all other requests
        _ => {
            println!("Unknown route requested: {}", req.uri().path());
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not Found"))
                .unwrap();
            Ok(response)
        }
    }
}

#[tokio::main]
async fn main() {
    // Configure the server address
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Starting mock TAP server on {}", addr);
    
    // Create a service function
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handle_request))
    });
    
    // Create and start the server
    let server = Server::bind(&addr).serve(make_svc);
    println!("Server started. Press Ctrl+C to stop.");
    
    // Run the server
    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}