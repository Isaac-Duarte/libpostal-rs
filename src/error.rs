//! Error types and handling for libpostal-rs.

/// Result type alias for libpostal operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for libpostal operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Initialization failed
    #[error("Failed to initialize libpostal: {message}")]
    InitializationFailed {
        /// Error message from libpostal
        message: String,
    },

    /// Data management errors
    #[error("Data error: {message}")]
    DataError {
        /// Error message
        message: String,
    },

    /// Parsing errors
    #[error("Parse error: {message}")]
    ParseError {
        /// Error message
        message: String,
    },

    /// Normalization errors
    #[error("Normalization error: {message}")]
    NormalizationError {
        /// Error message
        message: String,
    },

    /// FFI errors
    #[error("FFI error: {message}")]
    FfiError {
        /// Error message
        message: String,
    },

    /// I/O errors
    #[error("I/O error: {source}")]
    IoError {
        /// Source error
        #[from]
        source: std::io::Error,
    },

    /// Network errors (for data downloads)
    #[cfg(feature = "runtime-data")]
    #[error("Network error: {message}")]
    NetworkError {
        /// Error message
        message: String,
    },
}

impl Error {
    /// Create a new initialization error
    pub fn initialization_failed(message: impl Into<String>) -> Self {
        Self::InitializationFailed {
            message: message.into(),
        }
    }

    /// Create a new data error
    pub fn data_error(message: impl Into<String>) -> Self {
        Self::DataError {
            message: message.into(),
        }
    }

    /// Create a new parse error
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::ParseError {
            message: message.into(),
        }
    }

    /// Create a new normalization error
    pub fn normalization_error(message: impl Into<String>) -> Self {
        Self::NormalizationError {
            message: message.into(),
        }
    }

    /// Create a new FFI error
    pub fn ffi_error(message: impl Into<String>) -> Self {
        Self::FfiError {
            message: message.into(),
        }
    }

    /// Create a new network error
    #[cfg(feature = "runtime-data")]
    pub fn network_error(message: impl Into<String>) -> Self {
        Self::NetworkError {
            message: message.into(),
        }
    }
}
