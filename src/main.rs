use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
};
use url_shortener::{Service, Storage};

#[tokio::main]
async fn main() {
    let storage = Storage::new();
    let service = Arc::new(Service::new(storage));

    let app = Router::new()
        .route("/", post(get_code))
        .route("/length", get(length))
        .route("/{code}", get(get_url))
        .with_state(service);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_url(State(service): State<Arc<Service>>, Path(code): Path<u64>) -> impl IntoResponse {
    service.process_get(code)
}
async fn length(State(service): State<Arc<Service>>) -> impl IntoResponse {
    service.length()
}

async fn get_code(
    State(service): State<Arc<Service>>,
    Json(url): Json<url_shortener::Url>,
) -> impl IntoResponse {
    service.process_post(url)
}
