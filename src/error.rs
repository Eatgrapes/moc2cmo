use std::path::PathBuf;

/// An error produced while reading a MOC3 model or writing a CMO3 project.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The input is not a supported or structurally valid MOC3 file.
    #[error("invalid MOC3: {0}")]
    InvalidMoc3(String),

    /// The CMO3 project could not be encoded.
    #[error("invalid CMO3 project: {0}")]
    InvalidCmo3(String),

    /// A CAN3 animation project could not be decoded or encoded.
    #[error("invalid CAN3 project: {0}")]
    InvalidCan3(String),

    /// A Cubism CAFF archive could not be decoded or encoded.
    #[error("invalid CAFF archive: {0}")]
    InvalidCaff(String),

    /// A filesystem operation failed.
    #[error("failed to access {path}: {source}")]
    Io {
        /// The path being accessed.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
}

/// A result returned by `moc2cmo` operations.
pub type Result<T> = std::result::Result<T, Error>;
