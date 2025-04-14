mod service;
mod storage;

use serde::{Deserialize, Serialize};
pub use service::Service;
pub use storage::Storage;

#[derive(Serialize, Clone, Eq, PartialEq, Hash)]
pub struct Code(pub u64);

#[derive(Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct Url(pub String);

impl From<u64> for Code {
    fn from(value: u64) -> Self {
        Code(value)
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
