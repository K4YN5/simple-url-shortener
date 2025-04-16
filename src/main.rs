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

async fn shutdown_signal(service_for_shutdown: Arc<Service>) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>(); // On non-unix, just wait for ctrl_c

    tokio::select! {
        _ = ctrl_c => { log::info!("Received Ctrl+C signal.") },
        _ = terminate => { log::info!("Received terminate signal.") },
    }

    log::info!("Signal received, starting graceful shutdown...");
    // Now call your application-specific shutdown logic
    service_for_shutdown.graceful_shutdown().await;
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let service = Arc::new(Service::new().await);

    log::info!("App starting now!");
    let app = Router::new()
        .route("/", post(get_code))
        .route("/length", get(length))
        .route("/echo", post(echo))
        .route("/{code}", get(get_url))
        .with_state(service.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    log::info!("Server up!");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(service))
        .await
        .unwrap();
}

async fn get_url(State(service): State<Arc<Service>>, Path(code): Path<u64>) -> impl IntoResponse {
    service.process_get(code).await
}
async fn length(State(service): State<Arc<Service>>) -> impl IntoResponse {
    service.length().await
}
async fn echo(body: Bytes) -> impl IntoResponse {
    eprintln!("{body:?}");
    (StatusCode::OK, body)
}
async fn get_code(
    State(service): State<Arc<Service>>,
    Json(url): Json<url_shortener::Url>,
) -> impl IntoResponse {
    service.process_post(url).await
}
