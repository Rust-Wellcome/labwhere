use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

/// Database errors
pub struct DatabaseError {
    pub message: String,
}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message.to_string())
    }
}

impl Debug for DatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message.to_string())
    }
}

impl Error for DatabaseError {}
