mod cache;
mod db;
mod service;
mod storage;

use std::sync::Arc;

use serde::{Deserialize, Serialize};
pub use service::Service;

#[derive(Serialize, Clone, Eq, PartialEq, Hash)]
pub struct SeqId(pub i64);

#[derive(Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct Url(pub String);

impl From<i64> for SeqId {
    fn from(value: i64) -> Self {
        SeqId(value)
    }
}

impl From<&str> for Url {
    fn from(value: &str) -> Self {
        Url(value.to_string())
    }
}

impl From<String> for Url {
    fn from(value: String) -> Self {
        Url(value)
    }
}

impl From<Url> for String {
    fn from(value: Url) -> Self {
        value.0
    }
}

pub async fn shutdown_signal(service_for_shutdown: Arc<Service>) {
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
