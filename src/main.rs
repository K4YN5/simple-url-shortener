use std::sync::{Arc, Mutex};

use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
};
use url_shortener::{Service, Storage};

#[tokio::main]
async fn main() {
    let storage = Arc::new(Mutex::new(Storage::new()));

    let app = Router::new()
        .route("/", post(get_code))
        .route("/{code}", get(get_url))
        .with_state(storage);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_url(
    State(storage): State<Arc<Mutex<Storage>>>,
    Path(code): Path<u64>,
) -> impl IntoResponse {
    let storage = storage.lock().unwrap();

    Service::process_get(&storage, code)
}

async fn get_code(
    State(storage): State<Arc<Mutex<Storage>>>,
    Json(url): Json<url_shortener::Url>,
) -> impl IntoResponse {
    let mut storage = storage.lock().unwrap();

    Service::process_post(&mut storage, url)
}
