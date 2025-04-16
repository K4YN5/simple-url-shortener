use std::{
    hash::{DefaultHasher, Hasher},
    sync::Arc,
};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{Hash, Url, cache::CachableStorage, storage::Storages};

pub struct Service {
    storage: Arc<CachableStorage>,
}

#[allow(dead_code)]
impl Service {
    pub async fn new() -> Self {
        let storage = CachableStorage::new().await;
        Self {
            storage: Arc::new(storage),
        }
    }

    pub async fn process_post(&self, mut url: Url) -> Response {
        if !Self::is_valid_url(&mut url) {
            eprintln!("Invalid URL: {}", url.0);
            return (axum::http::StatusCode::NOT_FOUND, "Invalid URL").into_response();
        }

        if let Some(code) = self.storage.get_key_by_value(&url).await {
            return axum::Json(code).into_response();
        };

        let hash = Service::hash(&url);

        self.storage.insert(url, hash.clone()).await;

        axum::Json(hash).into_response()
    }

    pub async fn length(&self) -> Response {
        (StatusCode::OK, self.storage.length().await.to_string()).into_response()
    }

    pub async fn process_get(&self, code: u64) -> Response {
        match self.storage.get(code.into()).await {
            Some(url) => axum::response::Redirect::permanent(&url.0).into_response(),
            None => (
                axum::http::StatusCode::NOT_FOUND,
                "URL not found in our system",
            )
                .into_response(),
        }
    }

    pub fn is_valid_url(url: &mut Url) -> bool {
        if url.0.starts_with("http://") {
            url.0 = (url.0[7..]).to_string();
        } else if url.0.starts_with("https://") {
            url.0 = (url.0[8..]).to_string();
        } else {
            return false;
        }

        let (host_port, path_query) = match url.0.find('/') {
            Some(i) => (&url.0[..i], &url.0[i..]),
            None => (url.0.as_str(), ""),
        };

        let host = match host_port.find(':') {
            Some(i) => &host_port[..i],
            None => host_port,
        };

        let host = host.to_ascii_lowercase();

        if host == "localhost" || host.ends_with(".local") {
            return false;
        }

        let parts: Vec<&str> = host.split('.').collect();
        if parts.len() < 2 || parts.last().unwrap().len() < 2 {
            return false;
        }

        let clean_host = match host.strip_prefix("www.") {
            Some(host) => host,
            None => &host,
        };

        url.0 = format!("{clean_host}{path_query}");
        true
    }

    pub fn hash(url: &Url) -> Hash {
        let mut hasher = DefaultHasher::new();
        std::hash::Hash::hash(&url, &mut hasher);
        Hash(hasher.finish())
    }

    pub async fn graceful_shutdown(&self) {
        log::trace!("Server shutting down...");
        self.storage.shutdown().await;
        log::info!("Server shut down completely!");
    }
}
