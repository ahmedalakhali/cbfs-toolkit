use std::fmt;
use std::io;

/// Application-level error types.
/// These are infrastructure-agnostic to maintain clean architecture.
#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    Database(String),  // Generic database error message
    Vault { code: i32, message: String },  // Vault error with code and description
    Utf8(std::string::FromUtf8Error),
    Auth(String),
    General(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(err) => write!(f, "IO Error: {}", err),
            AppError::Database(msg) => write!(f, "Database Error: {}", msg),
            AppError::Vault { code, message } => write!(f, "Vault Error ({}): {}", code, message),
            AppError::Utf8(err) => write!(f, "UTF-8 Conversion Error: {}", err),
            AppError::Auth(msg) => write!(f, "Authentication Error: {}", msg),
            AppError::General(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<std::string::FromUtf8Error> for AppError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        AppError::Utf8(err)
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::General(err)
    }
}

impl From<&str> for AppError {
    fn from(err: &str) -> Self {
        AppError::General(err.to_string())
    }
}

// Infrastructure error conversions - these convert at the boundary
// keeping the Domain free of specific framework dependencies.
// The type itself is not stored, only the message.

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<cbfsvault::CBFSVaultError> for AppError {
    fn from(err: cbfsvault::CBFSVaultError) -> Self {
        AppError::Vault { 
            code: err.get_code(), 
            message: format!("Vault operation failed (code {})", err.get_code())
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
