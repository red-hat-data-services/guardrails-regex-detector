use std::{env, net::{SocketAddr, IpAddr}};
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
mod detectors;
use detectors::handle_text_contents;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();
    
    // Get port from environment variable or use default
    let mut http_port = 8080;
    if let Ok(port) = env::var("HTTP_PORT") {
        match port.parse::<u16>() {
            Ok(port) => http_port = port,
            Err(err) => println!("{}", err),
        }
    }

    // Get host from environment variable or use default
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    
    let app = Router::new()
        .route("/health", get(|| async { "Hello, World!" }))
        .route("/api/v1/text/contents", post(handle_text_contents))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    let ip: IpAddr = host.parse().expect("Failed to parse host IP address");
    let addr = SocketAddr::from((ip, http_port));
    
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}