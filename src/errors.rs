use strum_macros::Display;
use thiserror::Error;

use venum::{errors_result::VenumError, value::Value, value_type::ValueType};

#[derive(Debug, Display, PartialEq)]
pub enum WrappedErrors {
    VenumError(VenumError),
}

#[derive(Debug, PartialEq, Eq, Display)]
pub enum DataAccessErrors {
    IllegalIdxAccess { idx: usize },
    IllegalNameAccess { name: String },
}

#[derive(Error, Debug, PartialEq)]
#[error("error: {msg:?}; problem value: {src_val:?}. Details: {details:?}")]
pub struct SplitError {
    msg: String,
    src_val: Value,
    details: Option<String>,
}
impl SplitError {
    pub fn new(msg: String, src_val: Value) -> Self {
        Self {
            msg,
            src_val,
            details: None,
        }
    }
    pub fn new_with_details(msg: String, src_val: Value, details: Option<String>) -> Self {
        Self {
            msg,
            src_val,
            details,
        }
    }
}

#[derive(Debug, PartialEq, Display)]
pub enum TransformErrors {
    Generic { msg: String },
    Split(SplitError),
}

#[derive(Debug, Display, PartialEq, Eq)]
pub enum ContainerOpsErrors {
    Generic {
        msg: String,
    },
    SplitItemError {
        idx: usize,
        msg: String,
    },
    ConvertAsError {
        src_idx: usize,
        src_data_type: ValueType,
        target_data_type: ValueType,
        details: Option<String>,
    },
}

#[derive(Debug, PartialEq, Display)]
pub enum VenumTdsError {
    Generic { msg: String },
    Wrapped(WrappedErrors),
    DataAccess(DataAccessErrors),
    Transform(TransformErrors),
    ContainerOps(ContainerOpsErrors),
}

pub type Result<T> = std::result::Result<T, VenumTdsError>;

impl From<VenumError> for VenumTdsError {
    fn from(ve: VenumError) -> Self {
        VenumTdsError::Wrapped(WrappedErrors::VenumError(ve))
    }
}
