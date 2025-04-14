use std::hash::{DefaultHasher, Hash, Hasher};

use axum::response::{IntoResponse, Response};

use crate::{Code, Storage, Url};

pub struct Service {}

impl Service {
    pub fn process_post(storage: &mut Storage, url: Url) -> Response {
        if !Self::is_strict_valid_url(&url.0) {
            return (axum::http::StatusCode::NOT_ACCEPTABLE, "Invalid URL").into_response();
        }
        let code: Code = if let Some(code) = storage.inverted_get(&url.0) {
            code
        } else {
            storage.insert(&url.0)
        };

        axum::Json(code).into_response()
    }

    pub fn process_get(storage: &Storage, code: u64) -> Response {
        match storage.get(code) {
            Some(url) => axum::response::Redirect::permanent(&url.0).into_response(),
            None => (
                axum::http::StatusCode::NOT_FOUND,
                "URL not found in our system",
            )
                .into_response(),
        }
    }

    fn is_strict_valid_url(s: &str) -> bool {
        if let Ok(url) = url::Url::parse(s) {
            matches!(url.scheme(), "http" | "https")
                && url.has_host()
                && url.host_str().map(|h| h.contains('.')).unwrap_or(false)
        } else {
            false
        }
    }

    pub fn hash(url: Url) -> Code {
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        Code(hasher.finish())
    }
}
