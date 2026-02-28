//! Authorizer-specific error type.

use std::fmt;

/// Errors that can occur during authorization.
#[derive(Debug)]
pub enum AuthError {
    /// The Authorization header is missing or malformed.
    MissingToken,
    /// The JWT failed signature or claims validation.
    InvalidToken(String),
    /// Failed to fetch or parse the JWKS endpoint.
    JwksError(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::MissingToken => write!(f, "Missing or malformed Authorization header"),
            AuthError::InvalidToken(msg) => write!(f, "Invalid token: {msg}"),
            AuthError::JwksError(msg) => write!(f, "JWKS error: {msg}"),
        }
    }
}

impl std::error::Error for AuthError {}
