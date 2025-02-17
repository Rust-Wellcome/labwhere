use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

// Dynamically adds the Display and Debug traits.
macro_rules! impl_error {
    ($err_type:ident) => {
        impl Display for $err_type {
            fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
                write!(f, "{}", self.message)
            }
        }

        impl Debug for $err_type {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.message)
            }
        }

        impl Error for $err_type {}
    };
}

pub struct NotFoundError {
    pub message: String,
}

pub struct BarcodeEmptyError {
    pub message: String,
}

impl_error!(NotFoundError);
impl_error!(BarcodeEmptyError);

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

/// Converts from `NotFoundError` into a `LabwhereError`
impl From<NotFoundError> for LabwhereError {
    fn from(err: NotFoundError) -> Self {
        LabwhereError::NotFoundError(err)
    }
}

/// Converts from `BarcodeEmptyError` into a `LabwhereError`
impl From<BarcodeEmptyError> for LabwhereError {
    fn from(err: BarcodeEmptyError) -> Self {
        LabwhereError::BarcodeEmptyError(err)
    }
}
