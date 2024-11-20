pub mod name_format_error;
pub mod not_found_error;
pub mod sql_error;

use crate::errors::not_found_error::NotFoundError;
use crate::errors::sql_error::DatabaseError;
use std::error::Error;
use std::fmt::Debug;

/// A generalised error for Labware
#[derive(Debug)]
pub enum LabwhereError {
    NotFound(NotFoundError),
    DatabaseError(DatabaseError),
}

impl std::fmt::Display for LabwhereError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LabwhereError::NotFound(err) => write!(f, "{}", err),
            LabwhereError::DatabaseError(err) => write!(f, "{}", err),
        }
    }
}

impl Error for LabwhereError {}

impl From<NotFoundError> for LabwhereError {
    fn from(err: NotFoundError) -> Self {
        LabwhereError::NotFound(err)
    }
}

impl From<DatabaseError> for LabwhereError {
    fn from(err: DatabaseError) -> Self {
        LabwhereError::DatabaseError(err)
    }
}
