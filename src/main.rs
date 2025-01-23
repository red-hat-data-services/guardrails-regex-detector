use std::{env, net::SocketAddr};

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

    let mut http_port = 8080;
    if let Ok(port) = env::var("HTTP_PORT") {
        match port.parse::<u16>() {
            Ok(port) => http_port = port,
            Err(err) => println!("{}", err),
        }
    }

    let app = Router::new()
        .route("/health", get(|| async { "Hello, World!" }))
        .route("/api/v1/text/contents", post(handle_text_contents))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], http_port));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
