use venum::venum::Value;

use crate::traits::{Indexed, Named, TypeInfo};

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
}

impl Named for DataCell {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }
    fn set_name(&mut self, name: &str) {
        self.name = String::from(name);
    }
}

impl TypeInfo for DataCell {
    fn get_type_info(&self) -> &Value {
        &self.type_info
    }
}

#[cfg(test)]
mod tests {
    use venum::venum::Value;

    use super::*;

    #[test]
    fn test_indexed_on_data_cell() {
        let d = DataCell::new_without_data(Value::bool_default(), String::from("col1"), 123);
        assert_eq!(123, d.get_idx());
    }

    #[test]
    fn test_named_on_data_cell() {
        let d = DataCell::new_without_data(Value::bool_default(), String::from("col1"), 123);
        assert_eq!("col1", d.get_name());
    }

    #[test]
    fn test_typeinfo_on_data_cell() {
        let d = DataCell::new_without_data(Value::bool_default(), String::from("col1"), 123);
        assert_eq!(&Value::bool_default(), d.get_type_info());
    }
}
