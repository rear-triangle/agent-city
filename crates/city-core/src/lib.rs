pub mod archive;
pub mod codex;
pub mod domain;
pub mod files;
pub mod provider;
pub mod resources;
pub mod retention;
pub mod runtime;
pub mod skills;
pub mod store;
pub use domain::*;
pub use runtime::Runtime;

pub type Result<T> = std::result::Result<T, CityError>;

#[derive(Debug, thiserror::Error, serde::Serialize)]
#[serde(rename_all = "camelCase")]
#[error("{code}: {message}")]
pub struct CityError {
    pub code: String,
    pub message: String,
}
impl CityError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}
impl From<rusqlite::Error> for CityError {
    fn from(e: rusqlite::Error) -> Self {
        Self::new("storage_unavailable", e.to_string())
    }
}
impl From<std::io::Error> for CityError {
    fn from(e: std::io::Error) -> Self {
        Self::new("io_error", e.to_string())
    }
}
impl From<serde_json::Error> for CityError {
    fn from(e: serde_json::Error) -> Self {
        Self::new("invalid_data", e.to_string())
    }
}
impl From<reqwest::Error> for CityError {
    fn from(e: reqwest::Error) -> Self {
        Self::new("provider_unavailable", e.without_url().to_string())
    }
}
