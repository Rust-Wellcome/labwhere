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
    NotFoundError(NotFoundError),
    BarcodeEmptyError(BarcodeEmptyError),
}

impl LabwhereError {
    pub fn not_found_error(entity: &str) -> LabwhereError {
        LabwhereError::NotFoundError(NotFoundError {
            message: format!("{} not found!", entity),
        })
    }

    pub fn barcode_empty_error() -> LabwhereError {
        LabwhereError::BarcodeEmptyError(BarcodeEmptyError {
            message: "Barcode is empty!".to_string(),
        })
    }
}

impl From<NotFoundError> for LabwhereError {
    fn from(err: NotFoundError) -> Self {
        LabwhereError::NotFoundError(err)
    }
}

impl From<BarcodeEmptyError> for LabwhereError {
    fn from(err: BarcodeEmptyError) -> Self {
        LabwhereError::BarcodeEmptyError(err)
    }
}
