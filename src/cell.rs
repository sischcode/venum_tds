use venum::venum::Value;

use crate::errors::{DataAccessErrors, Result, VenumTdsError};
use crate::traits::{Indexed, Named};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct DataCell {
    pub type_info: Value, // We use the enum variants default value as our type info
    pub name: String,     // the column header
    pub idx: usize,       // columns are zero-indexed for now!
    pub data: Option<Value>, // Data
}

impl DataCell {
    pub fn new_without_data(type_info: Value, name: String, idx: usize) -> Self {
        Self {
            type_info,
            name,
            idx,
            data: None,
        }
    }

    pub fn new(type_info: Value, name: String, idx: usize, data: Option<Value>) -> Self {
        Self {
            type_info,
            name,
            idx,
            data,
        }
    }
}

impl Indexed for DataCell {
    fn get_idx(&self) -> usize {
        self.idx
    }
    fn set_idx(&mut self, idx: usize) {
        self.idx = idx;
    }
    fn get_by_idx(&self, idx: usize) -> Result<&Self> {
        match idx == self.idx {
            true => Ok(self),
            false => Err(VenumTdsError::DataAccess(
                DataAccessErrors::IllegalIdxAccess { idx },
            )),
        }
    }
    fn get_by_idx_mut(&mut self, idx: usize) -> Result<&mut Self> {
        match idx == self.idx {
            true => Ok(self),
            false => Err(VenumTdsError::DataAccess(
                DataAccessErrors::IllegalIdxAccess { idx },
            )),
        }
    }
}

impl Named for DataCell {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }
    fn set_name(&mut self, name: &str) {
        self.name = String::from(name);
    }
    fn get_by_name(&self, name: &str) -> Result<&Self> {
        match name == self.name {
            true => Ok(self),
            false => Err(VenumTdsError::DataAccess(
                DataAccessErrors::IllegalNameAccess {
                    name: String::from(name),
                },
            )),
        }
    }
    fn get_by_name_mut(&mut self, name: &str) -> Result<&mut Self> {
        match name == self.name {
            true => Ok(self),
            false => Err(VenumTdsError::DataAccess(
                DataAccessErrors::IllegalNameAccess {
                    name: String::from(name),
                },
            )),
        }
    }
}
