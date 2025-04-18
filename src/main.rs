use std::sync::Arc;

use axum::{
    Json, Router,
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};

use url_shortener::Service;

#[tokio::main]
async fn main() {
    env_logger::init();
    let service = Arc::new(Service::new().await);

    log::info!("App starting now!");
    let app = Router::new()
        .route("/", post(get_id))
        .route("/length", get(length))
        .route("/echo", post(echo))
        .route("/{id}", get(get_url))
        .with_state(service.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    log::info!("Server up!");
    axum::serve(listener, app)
        .with_graceful_shutdown(url_shortener::shutdown_signal(service))
        .await
        .unwrap();
}

async fn get_url(State(service): State<Arc<Service>>, Path(id): Path<String>) -> impl IntoResponse {
    service.process_get(id).await
}
async fn length(State(service): State<Arc<Service>>) -> impl IntoResponse {
    service.length().await
}
async fn echo(body: Bytes) -> impl IntoResponse {
    log::debug!("{body:?}");
    (StatusCode::OK, body)
}
async fn get_id(
    State(service): State<Arc<Service>>,
    Json(url): Json<url_shortener::Url>,
) -> impl IntoResponse {
    service.process_post(url).await
}
