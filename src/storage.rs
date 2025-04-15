#![allow(clippy::new_without_default)]
use std::sync::Arc;

use dashmap::DashMap;

use crate::{Code, Service, Url};

#[derive(Clone)]
pub struct Storage {
    code_to_url: DashMap<Code, Arc<Url>>,
    url_to_code: DashMap<Arc<Url>, Code>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            code_to_url: DashMap::new(),
            url_to_code: DashMap::new(),
        }
    }

    pub fn get(&self, code: u64) -> Option<Url> {
        let url_ref = self.code_to_url.get(&code.into())?;
        Some((**url_ref).clone())
    }

    pub fn length(&self) -> usize {
        self.code_to_url.len()
    }

    pub fn insert(&self, url: &str) -> Code {
        let url: Url = url.to_string().into();

        let code = Service::hash(url.clone());

        let url = Arc::new(url);

        self.code_to_url.insert(code.clone(), url.clone());
        self.url_to_code.insert(url, code.clone());

        code
    }

    pub fn inverted_get(&self, url: &str) -> Option<Code> {
        let url: Url = url.into();
        self.url_to_code.get(&url).map(|r| r.clone())
    }
}
