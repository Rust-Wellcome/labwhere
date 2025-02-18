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

// Dynamically adds From traits converting custom error types to global `LabwhereError` enum.
macro_rules! impl_error_conversion {
    ($err_type:ident) => {
        impl From<$err_type> for LabwhereError {
            fn from(err: $err_type) -> Self {
                LabwhereError::$err_type(err)
            }
        }
    };
}

pub struct NotFoundError {
    pub message: String,
}

pub struct BarcodeEmptyError {
    pub message: String,
}
pub struct NameFormatError {
    pub message: String,
}

impl_error!(NotFoundError);
impl_error!(BarcodeEmptyError);
impl_error!(NameFormatError);

/// A generalised error for Labware
#[derive(Debug)]
pub enum LabwhereError {
    NotFoundError(NotFoundError),
    BarcodeEmptyError(BarcodeEmptyError),
    NameFormatError(NameFormatError),
}

impl_error_conversion!(NotFoundError);
impl_error_conversion!(BarcodeEmptyError);
impl_error_conversion!(NameFormatError);

/// As we have implemented the `From` trait on each of our error types
/// that converts them into Labwhere error, we can invoke `into()` on each
/// of the sub error types which will convert them to `LabwhereError`.
impl LabwhereError {
    pub fn not_found_error(entity: &str) -> LabwhereError {
        (NotFoundError {
            message: format!("{} not found!", entity),
        })
        .into()
    }

    pub fn barcode_empty_error() -> LabwhereError {
        (BarcodeEmptyError {
            message: "Barcode is empty!".to_string(),
        })
        .into()
    }

    pub fn name_format_error() -> LabwhereError {
        (NameFormatError {
            message: "Invalid name format!".to_string(),
        })
        .into()
    }
}
