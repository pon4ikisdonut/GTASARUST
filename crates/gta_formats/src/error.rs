use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FormatError {
    #[error("I/O error on '{path}': {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("unexpected EOF at offset {offset} (needed {needed} bytes, remaining {remaining})")]
    UnexpectedEof {
        offset: usize,
        needed: usize,
        remaining: usize,
    },
    #[error("invalid magic header: expected '{expected}', got '{actual}'")]
    InvalidMagic { expected: String, actual: String },
    #[error("unsupported format or version: {0}")]
    Unsupported(String),
    #[error("corrupt or malformed record at {context}: {detail}")]
    Malformed { context: String, detail: String },
    #[error("entry not found in archive: '{0}'")]
    EntryNotFound(String),
}
