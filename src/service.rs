use std::sync::Arc;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{Url, cache::CachableStorage, storage::Storages};

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
        if !Self::normalize_and_validate_url(&mut url) {
            log::trace!("Invalid url {}", url.0);
            return (axum::http::StatusCode::NOT_FOUND, "Invalid URL").into_response();
        }

        if let Some(id) = self.storage.get_key_by_value(&url).await {
            return axum::Json(id).into_response();
        };

        let numerical_id = self.storage.insert(url.clone()).await;
        let id = base62::encode(numerical_id.0 as u32);

        axum::Json(id).into_response()
    }

    pub async fn length(&self) -> Response {
        (StatusCode::OK, self.storage.length().await.to_string()).into_response()
    }

    pub async fn process_get(&self, id: String) -> Response {
        let numerical_id = match base62::decode(id) {
            Ok(id_u128) => crate::SeqId(id_u128.try_into().unwrap()),
            Err(_) => {
                return (axum::http::StatusCode::NOT_FOUND, "Invalid id").into_response();
            }
        };

        match self.storage.get(numerical_id).await {
            Some(url) => {
                axum::response::Redirect::permanent(&format!("https://{}", url.0)).into_response()
            }
            None => (
                axum::http::StatusCode::NOT_FOUND,
                "URL not found in our system",
            )
                .into_response(),
        }
    }

    pub fn normalize_and_validate_url(url: &mut Url) -> bool {
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

    pub async fn graceful_shutdown(&self) {
        log::trace!("Server shutting down...");
        self.storage.shutdown().await;
        log::info!("Server shut down completely!");
    }
}
