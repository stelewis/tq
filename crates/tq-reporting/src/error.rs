use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReportingError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
