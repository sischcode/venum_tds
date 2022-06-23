use strum_macros::Display;

use venum::errors::VenumError;

#[derive(Debug, PartialEq, Display, Clone)]
pub enum WrappedErrors {
    VenumError(VenumError),
}

#[derive(Debug, PartialEq, Display, Clone)]
pub enum VenumTdsError {
    Generic { msg: String },
    Wrapped(WrappedErrors),
}

pub type Result<T> = std::result::Result<T, VenumTdsError>;

impl From<VenumError> for VenumTdsError {
    fn from(ve: VenumError) -> Self {
        VenumTdsError::Wrapped(WrappedErrors::VenumError(ve))
    }
}
