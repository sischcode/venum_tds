use venum::value::Value;
use venum::value_type::ValueType;

use crate::errors::{Result, VenumTdsError};

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct DataCell {
    pub dtype: ValueType, // We use the enum variants default value as our type info
    pub name: String,     // the "column header"
    pub idx: usize,       // "columns" are zero-indexed for now!
    pub data: Value,      // Data
}

impl DataCell {
    pub fn new_with_type_info(
        type_info: ValueType,
        name: String,
        idx: usize,
        data: Value,
    ) -> Result<Self> {
        if data.is_none() || type_info == ValueType::try_from(&data).unwrap() {
            Ok(Self {
                dtype: type_info,
                name,
                idx,
                data,
            })
        } else {
            Err(VenumTdsError::Generic { msg: format!("Cannot create DataCell. Type info provided ({}) and Type info, inferred from Value type ({}) do not match!", type_info, ValueType::try_from(data).unwrap()) })
        }
    }

    pub fn new(name: String, idx: usize, data: Value) -> Result<Self> {
        if data.is_none() {
            Err(VenumTdsError::Generic {
                msg: format!(
                    "Cannot create DataCell. Cannot infer Type info from Value type ({})!",
                    data
                ),
            })
        } else {
            // we can safely unwrap, we checked it above (only Value::None is not "intoable")
            Ok(Self {
                dtype: ValueType::try_from(&data).unwrap(),
                name,
                idx,
                data,
            })
        }
    }

    pub fn new_without_data(type_info: ValueType, name: String, idx: usize) -> Self {
        Self {
            dtype: type_info,
            name,
            idx,
            data: Value::None,
        }
    }

    pub fn get_type_info(&self) -> &ValueType {
        &self.dtype
    }
    pub fn set_type_info(&mut self, type_info: ValueType) {
        self.dtype = type_info;
    }

    pub fn get_idx(&self) -> usize {
        self.idx
    }
    pub fn set_idx(&mut self, idx: usize) {
        self.idx = idx;
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }
    pub fn set_name(&mut self, name: &str) {
        self.name = String::from(name);
    }

    pub fn get_data(&self) -> &Value {
        &self.data
    }
    pub fn get_data_mut(&mut self) -> &mut Value {
        &mut self.data
    }
    pub fn set_data(&mut self, data: Value) {
        self.data = data;
    }
}

#[cfg(test)]
mod tests {
    // TODO
}
