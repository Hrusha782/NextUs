use crate::limits::ValidationError;
use crate::parser::ParseError;
use crate::runtime::RuntimeError;
use std::fmt;

/// Unified error type for all NextUs operations
#[derive(Debug)]
pub enum NextUsError {
    Parse(ParseError),
    Validation(ValidationError),
    Runtime(RuntimeError),
    Io(std::io::Error),
}

impl fmt::Display for NextUsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NextUsError::Parse(e) => write!(f, "Parse error: {}", e),
            NextUsError::Validation(e) => write!(f, "Validation error: {}", e),
            NextUsError::Runtime(e) => write!(f, "Runtime error: {:?}", e),
            NextUsError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for NextUsError {}

impl From<ParseError> for NextUsError {
    fn from(e: ParseError) -> Self {
        NextUsError::Parse(e)
    }
}

impl From<ValidationError> for NextUsError {
    fn from(e: ValidationError) -> Self {
        NextUsError::Validation(e)
    }
}

impl From<RuntimeError> for NextUsError {
    fn from(e: RuntimeError) -> Self {
        NextUsError::Runtime(e)
    }
}

impl From<std::io::Error> for NextUsError {
    fn from(e: std::io::Error) -> Self {
        NextUsError::Io(e)
    }
}
