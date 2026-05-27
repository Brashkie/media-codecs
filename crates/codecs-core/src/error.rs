//! Error types for `mcd-core`.
//!
//! All codec errors derive from [`CodecError`] and carry a discriminant
//! [`CodecErrorKind`] for FFI-safe identification.

use std::fmt;
use thiserror::Error;

/// Discriminant of a [`CodecError`] — string-safe for FFI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CodecErrorKind {
    /// Codec name/id not found in the registry.
    NotFound,
    /// Codec exists but cannot encode/decode this configuration.
    Unsupported,
    /// Input data is malformed (truncated, invalid header, etc.).
    InvalidData,
    /// Output buffer too small.
    BufferTooSmall,
    /// Encoder/decoder is in a bad state (e.g. flushed, closed).
    InvalidState,
    /// Operation was cancelled (AbortSignal).
    Cancelled,
    /// Internal bug — should never happen.
    Internal,
}

impl CodecErrorKind {
    /// Return the kind as a stable kebab-case string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            CodecErrorKind::NotFound => "not_found",
            CodecErrorKind::Unsupported => "unsupported",
            CodecErrorKind::InvalidData => "invalid_data",
            CodecErrorKind::BufferTooSmall => "buffer_too_small",
            CodecErrorKind::InvalidState => "invalid_state",
            CodecErrorKind::Cancelled => "cancelled",
            CodecErrorKind::Internal => "internal",
        }
    }
}

impl fmt::Display for CodecErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Unified codec error type.
///
/// Carries a [`CodecErrorKind`] discriminant, a human-readable message,
/// optional context (codec name, function name, etc.), and a `source`
/// for error chaining.
#[derive(Debug, Error)]
pub struct CodecError {
    kind: CodecErrorKind,
    message: String,
    context: Option<String>,
    #[source]
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ctx) = &self.context {
            write!(f, "[{}] {} ({})", self.kind, self.message, ctx)
        } else {
            write!(f, "[{}] {}", self.kind, self.message)
        }
    }
}

impl CodecError {
    /// Build a new error with the given kind + message.
    pub fn new(kind: CodecErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            context: None,
            source: None,
        }
    }

    /// Attach a context string (e.g. the codec name).
    #[must_use]
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Attach an underlying source error.
    #[must_use]
    pub fn with_source<E: std::error::Error + Send + Sync + 'static>(mut self, source: E) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// Get the error kind.
    #[must_use]
    pub const fn kind(&self) -> CodecErrorKind {
        self.kind
    }

    /// Get the error message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get the optional context.
    #[must_use]
    pub fn context(&self) -> Option<&str> {
        self.context.as_deref()
    }

    // ── Factories ──────────────────────────────────────────────────────────

    /// Codec was not found in the registry.
    pub fn not_found(name: impl Into<String>) -> Self {
        let name = name.into();
        Self::new(CodecErrorKind::NotFound, format!("codec not found: {name}"))
            .with_context(name)
    }

    /// Codec exists but doesn't support this configuration.
    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new(CodecErrorKind::Unsupported, message)
    }

    /// Input data malformed.
    pub fn invalid_data(message: impl Into<String>) -> Self {
        Self::new(CodecErrorKind::InvalidData, message)
    }

    /// Output buffer too small.
    pub fn buffer_too_small(needed: usize, got: usize) -> Self {
        Self::new(
            CodecErrorKind::BufferTooSmall,
            format!("output buffer too small: need {needed} bytes, got {got}"),
        )
    }

    /// Encoder/decoder is in an invalid state.
    pub fn invalid_state(message: impl Into<String>) -> Self {
        Self::new(CodecErrorKind::InvalidState, message)
    }

    /// Operation cancelled (AbortSignal).
    pub fn cancelled() -> Self {
        Self::new(CodecErrorKind::Cancelled, "operation cancelled")
    }

    /// Internal bug.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(CodecErrorKind::Internal, message)
    }
}

/// Convenience `Result` alias.
pub type CodecResult<T> = Result<T, CodecError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_strings_stable() {
        assert_eq!(CodecErrorKind::NotFound.as_str(), "not_found");
        assert_eq!(CodecErrorKind::InvalidData.as_str(), "invalid_data");
    }

    #[test]
    fn error_display_with_context() {
        let e = CodecError::not_found("opus");
        let s = format!("{e}");
        assert!(s.contains("not_found"));
        assert!(s.contains("opus"));
    }

    #[test]
    fn error_chain() {
        let inner = std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "eof");
        let e = CodecError::invalid_data("truncated stream").with_source(inner);
        assert_eq!(e.kind(), CodecErrorKind::InvalidData);
        assert!(std::error::Error::source(&e).is_some());
    }

    #[test]
    fn buffer_too_small_includes_sizes() {
        let e = CodecError::buffer_too_small(100, 50);
        let s = e.message();
        assert!(s.contains("100"));
        assert!(s.contains("50"));
    }
}
