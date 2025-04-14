#![allow(clippy::new_without_default)]
use std::{collections::HashMap, sync::Arc};

use crate::{Code, Service, Url};

// For now i will only use a std hashmap and everything in memory, later i will use a custom
// hashmap function and also a sqlite database for much larger data.
#[derive(Clone)]
pub struct Storage {
    /// Stores the short codes to long codes for GET
    code_to_url: HashMap<Code, Arc<Url>>,

    /// Stores the long codes to short code for when using POST to shorten a url find duplicates
    url_to_code: HashMap<Arc<Url>, Code>,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            code_to_url: HashMap::new(),
            url_to_code: HashMap::new(),
        }
    }

    pub fn get(&self, code: u64) -> Option<Url> {
        let url_ref = self.code_to_url.get(&code.into())?;
        Some((**url_ref).clone())
    }

    pub fn insert(&mut self, url: &str) -> Code {
        let url: Url = url.to_string().into();

        let code = Service::hash(url.clone());

        let url = Arc::new(url);

        self.code_to_url.insert(code.clone(), url.clone());
        self.url_to_code.insert(url, code.clone());

        code
    }

    pub fn inverted_get(&self, url: &str) -> Option<Code> {
        let url: Url = url.into();
        self.url_to_code.get(&url).cloned()
    }
}
