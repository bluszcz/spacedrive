//! Error types for BRAW operations
//!
//! This module defines comprehensive error types for BRAW file processing,
//! including SDK-specific errors, validation errors, and recovery strategies.

use thiserror::Error;


/// Comprehensive error type for BRAW operations
#[derive(Error, Debug)]
pub enum BrawError {
    /// Failed to open or read BRAW file
    #[error("Failed to open BRAW file: {0}")]
    OpenFailed(String),

    /// BlackmagicRAW SDK not available or not initialized
    #[error("BlackmagicRAW SDK not available - ensure SDK is installed and feature 'with-sdk' is enabled")]
    SdkUnavailable,

    /// SDK initialization failed
    #[error("SDK initialization failed with error code: {0}")]
    SdkInitializationFailed(i32),

    /// Generic SDK error with error code
    #[error("SDK operation failed with error code: {0}")]
    SdkError(i32),

    /// Invalid file format or corrupted file
    #[error("Invalid BRAW file format or corrupted file")]
    InvalidFormat,

    /// File too large for processing
    #[error("File too large: {size} bytes (max: {max_size} bytes)")]
    FileTooLarge { size: u64, max_size: u64 },

    /// Invalid file path
    #[error("Invalid file path - contains null bytes or invalid characters")]
    InvalidPath(std::path::PathBuf),

    /// Frame index out of range
    #[error("Frame {frame} out of range (0-{max_frames})")]
    FrameOutOfRange { frame: u32, max_frames: u32 },

    /// Unsupported feature or operation
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),

    /// Task join error for async operations
    #[error("Async task failed: {0}")]
    TaskJoinError(String),

    /// Image processing error
    #[error("Image processing error: {0}")]
    ImageProcessing(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Metadata parsing error
    #[error("Failed to parse metadata: {0}")]
    MetadataParsing(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Memory allocation error
    #[error("Memory allocation failed: {0}")]
    MemoryAllocation(String),

    /// Timeout error for long-running operations
    #[error("Operation timed out after {timeout_secs} seconds")]
    Timeout { timeout_secs: u64 },

    /// Hardware/GPU error
    #[error("Hardware error: {0}")]
    Hardware(String),

    /// Color processing error
    #[error("Color processing error: {0}")]
    ColorProcessing(String),

    /// File is not a valid BRAW file
    #[error("Invalid BRAW file format: {0}")]
    InvalidBrawFile(String),
}

impl BrawError {
    /// Check if the error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            // Permanent errors - not recoverable
            BrawError::SdkUnavailable
            | BrawError::InvalidFormat
            | BrawError::InvalidPath(_)
            | BrawError::UnsupportedOperation(_)
            | BrawError::Configuration(_) => false,

            // Temporary errors - might be recoverable
            BrawError::OpenFailed(_)
            | BrawError::SdkInitializationFailed(_)
            | BrawError::SdkError(_)
            | BrawError::Io(_)
            | BrawError::Timeout { .. }
            | BrawError::Hardware(_)
            | BrawError::MemoryAllocation(_)
            | BrawError::TaskJoinError(_) => true,

            // Context-dependent
            BrawError::FileTooLarge { .. } => false, // File won't get smaller
            BrawError::FrameOutOfRange { .. } => false, // Invalid request
            BrawError::ImageProcessing(_) => true, // Might work with different settings
            BrawError::MetadataParsing(_) => true, // Might be transient
            BrawError::ColorProcessing(_) => true, // Might work with different parameters
            BrawError::InvalidBrawFile(_) => false, // Permanent error
        }
    }

    /// Get suggested retry delay in milliseconds for recoverable errors
    pub fn retry_delay_ms(&self) -> Option<u64> {
        if !self.is_recoverable() {
            return None;
        }

        match self {
            BrawError::Io(_) => Some(100),
            BrawError::SdkError(_) | BrawError::SdkInitializationFailed(_) => Some(500),
            BrawError::Hardware(_) => Some(1000),
            BrawError::MemoryAllocation(_) => Some(200),
            BrawError::TaskJoinError(_) => Some(100),
            BrawError::Timeout { .. } => Some(2000),
            _ => Some(500),
        }
    }

    /// Get the error category for logging and monitoring
    pub fn category(&self) -> ErrorCategory {
        match self {
            BrawError::SdkUnavailable
            | BrawError::SdkInitializationFailed(_)
            | BrawError::SdkError(_) => ErrorCategory::Sdk,

            BrawError::InvalidFormat
            | BrawError::InvalidPath(_)
            | BrawError::FileTooLarge { .. } => ErrorCategory::FileValidation,

            BrawError::FrameOutOfRange { .. }
            | BrawError::UnsupportedOperation(_) => ErrorCategory::InvalidRequest,

            BrawError::ImageProcessing(_)
            | BrawError::ColorProcessing(_) => ErrorCategory::Processing,

            BrawError::Io(_) => ErrorCategory::Io,

            BrawError::MemoryAllocation(_)
            | BrawError::Hardware(_) => ErrorCategory::System,

            BrawError::Timeout { .. }
            | BrawError::TaskJoinError(_) => ErrorCategory::Runtime,

            BrawError::MetadataParsing(_)
            | BrawError::Configuration(_) => ErrorCategory::Configuration,

            BrawError::OpenFailed(_) => ErrorCategory::FileAccess,

            BrawError::InvalidBrawFile(_) => ErrorCategory::FileValidation,
        }
    }
}

/// Error categories for monitoring and handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// SDK-related errors
    Sdk,
    /// File validation errors
    FileValidation,
    /// Invalid API requests
    InvalidRequest,
    /// Image/video processing errors
    Processing,
    /// I/O related errors
    Io,
    /// System/hardware errors
    System,
    /// Runtime/async errors
    Runtime,
    /// Configuration errors
    Configuration,
    /// File access errors
    FileAccess,
}

/// Result type for BRAW operations
pub type BrawResult<T> = Result<T, BrawError>;

/// Create a standardized error for unsupported operations
pub fn unsupported_operation(operation: &str) -> BrawError {
    BrawError::UnsupportedOperation(format!("Operation '{}' is not supported", operation))
}

/// Create a standardized error for configuration issues
pub fn configuration_error(message: &str) -> BrawError {
    BrawError::Configuration(message.to_string())
}

/// Create a standardized timeout error
pub fn timeout_error(timeout_secs: u64) -> BrawError {
    BrawError::Timeout { timeout_secs }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_recoverability() {
        assert!(!BrawError::SdkUnavailable.is_recoverable());
        assert!(!BrawError::InvalidFormat.is_recoverable());
        assert!(BrawError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test")).is_recoverable());
        assert!(BrawError::SdkError(1).is_recoverable());
    }

    #[test]
    fn test_retry_delays() {
        assert_eq!(BrawError::SdkUnavailable.retry_delay_ms(), None);
        assert_eq!(BrawError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test")).retry_delay_ms(), Some(100));
        assert_eq!(BrawError::Hardware("test".to_string()).retry_delay_ms(), Some(1000));
    }

    #[test]
    fn test_error_categories() {
        assert_eq!(BrawError::SdkUnavailable.category(), ErrorCategory::Sdk);
        assert_eq!(BrawError::InvalidFormat.category(), ErrorCategory::FileValidation);
        assert_eq!(BrawError::ImageProcessing("test".to_string()).category(), ErrorCategory::Processing);
    }

    #[test]
    fn test_error_constructors() {
        let err = unsupported_operation("test");
        assert!(matches!(err, BrawError::UnsupportedOperation(_)));

        let err = configuration_error("test config");
        assert!(matches!(err, BrawError::Configuration(_)));

        let err = timeout_error(30);
        assert!(matches!(err, BrawError::Timeout { timeout_secs: 30 }));
    }
}