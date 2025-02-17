use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub struct NotFoundError {
    pub message: String,
}

pub struct BarcodeEmptyError {
    pub message: String,
}

impl Display for NotFoundError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message.to_string())
    }
}

impl Debug for NotFoundError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message.to_string())
    }
}

impl Display for BarcodeEmptyError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message.to_string())
    }
}

impl Debug for BarcodeEmptyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message.to_string())
    }
}

impl Error for NotFoundError {}
impl Error for BarcodeEmptyError {}

/// A generalised error for Labware
#[derive(Debug)]
pub enum LabwhereError {
    NotFound(NotFoundError),
    BarcodeEmptyError(BarcodeEmptyError),
}
