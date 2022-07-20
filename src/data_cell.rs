use venum::venum::{Value, ValueType};

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct DataCell {
    pub dtype: ValueType, // We use the enum variants default value as our type info
    pub name: String,     // the "column header"
    pub idx: usize,       // "columns" are zero-indexed for now!
    pub data: Value,      // Data
}

impl DataCell {
    pub fn new(type_info: ValueType, name: String, idx: usize, data: Value) -> Self {
        Self {
            dtype: type_info,
            name,
            idx,
            data,
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
    use venum::venum::Value;

    use super::*;

    #[test]
    fn test_indexed_on_data_cell() {
        let d = DataCell::new(ValueType::Bool, String::from("col1"), 123, Value::None);
        assert_eq!(123, d.get_idx());
    }

    #[test]
    fn test_named_on_data_cell() {
        let d = DataCell::new(ValueType::Bool, String::from("col1"), 123, Value::None);
        assert_eq!("col1", d.get_name());
    }

    #[test]
    fn test_typeinfo_on_data_cell() {
        let d = DataCell::new(ValueType::Bool, String::from("col1"), 123, Value::None);
        assert_eq!(&ValueType::Bool, d.get_type_info());
    }

    #[test]
    fn test_data_on_data_cell() {
        let d = DataCell::new(ValueType::Bool, String::from("col1"), 123, Value::None);
        assert_eq!(&Value::None, d.get_data());
    }
}
