use std::fmt;

/// Why a request or an operation failed, in the three ways the JS backend's callers tell failures apart.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// An error the JS code threw with a `status` (`Object.assign(new Error(message), { status })`): the API answers
    /// it as a problem body whose `detail` is `message`.
    Status {
        /// HTTP status of the answer.
        status: u16,
        /// Message shown to the client.
        message: String,
        /// The `code` property some of these errors carry, such as `CAPTURE_BLOCKED`.
        code: Option<&'static str>,
    },
    /// A `TypeError` or `RangeError`: the API answers 400 `Invalid request data`, never showing the message.
    Invalid(String),
    /// Any other failure: the API rethrows it and the server answers 500.
    Internal(String),
}

/// Result of a fallible backend operation.
pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Error {
    /// An error with an HTTP status and the message the client sees.
    ///
    /// # Arguments
    /// * `status` - HTTP status the API answers with.
    /// * `message` - Problem detail shown to the client.
    pub fn status(status: u16, message: impl Into<String>) -> Self {
        Self::Status { status, message: message.into(), code: None }
    }

    /// Where the JS code threw a `TypeError` or `RangeError` on malformed input.
    ///
    /// # Arguments
    /// * `message` - What was wrong, for logs only.
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::Invalid(message.into())
    }

    /// An internal failure, answered with a 500.
    ///
    /// # Arguments
    /// * `message` - What failed, for logs only.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    /// The same error with a `code` property.
    ///
    /// # Arguments
    /// * `code` - Machine-readable code, such as `CAPTURE_BLOCKED`.
    #[must_use]
    pub fn with_code(self, code: &'static str) -> Self {
        match self {
            Self::Status { status, message, .. } => Self::Status { status, message, code: Some(code) },
            other => other,
        }
    }

    /// `String(error)` of the error the JS code would have thrown, as a server's 500 answer shows it.
    pub fn to_js_string(&self) -> String {
        match self {
            Self::Internal(text) => text.clone(),
            Self::Status { message, .. } => format!("Error: {message}"),
            Self::Invalid(text) if text.contains("Error: ") || text.contains("Error [") => text.clone(),
            Self::Invalid(text) => format!("TypeError: {text}"),
        }
    }

    /// The HTTP status the error carries, if it has one.
    pub fn http_status(&self) -> Option<u16> {
        match self {
            Self::Status { status, .. } => Some(*status),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Status { message, .. } | Self::Invalid(message) | Self::Internal(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for Error {}

impl From<mm_js::json::ParseError> for Error {
    /// `JSON.parse` throws a `SyntaxError`, which callers treat as an internal failure unless they catch it.
    fn from(error: mm_js::json::ParseError) -> Self {
        Self::Internal(format!("SyntaxError: {error}"))
    }
}

impl From<mm_js::uri::UriError> for Error {
    /// `decodeURIComponent` throws a `URIError`, which callers treat as an internal failure unless they catch it.
    fn from(error: mm_js::uri::UriError) -> Self {
        Self::Internal(format!("URIError: {error}"))
    }
}

impl From<mm_js::json::StringifyError> for Error {
    /// `JSON.stringify` throws a `TypeError` for a `BigInt`.
    fn from(error: mm_js::json::StringifyError) -> Self {
        Self::Invalid(error.0.into())
    }
}

impl From<mm_msgpack::DecodeError> for Error {
    /// The JS decoder throws a `RangeError` for truncated input and for bytes after the value.
    fn from(error: mm_msgpack::DecodeError) -> Self {
        match error {
            mm_msgpack::DecodeError::Insufficient => Self::Invalid(error.to_string()),
            mm_msgpack::DecodeError::Invalid(message) if message.starts_with("Extra ") => Self::Invalid(message),
            mm_msgpack::DecodeError::Invalid(message) => Self::Internal(message),
        }
    }
}

impl From<mm_msgpack::EncodeError> for Error {
    fn from(error: mm_msgpack::EncodeError) -> Self {
        Self::Internal(format!("SyntaxError: {error}"))
    }
}
