use venum::venum::Value;

use crate::errors::{DataAccessErrors, Result, VenumTdsError};

use super::data_cell::DataCell;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct DataCellRow(Vec<DataCell>);

impl DataCellRow {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl Default for DataCellRow {
    fn default() -> Self {
        Self::new()
    }
}

impl DataCellRow {
    pub fn get_by_idx(&self, idx: usize) -> Option<&DataCell> {
        self.0.iter().find(|&vec_elem| vec_elem.get_idx() == idx)
    }
    pub fn get_by_idx_mut(&mut self, idx: usize) -> Option<&mut DataCell> {
        self.0.iter_mut().find(|vec_elem| vec_elem.get_idx() == idx)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&DataCell> {
        self.0.iter().find(|&vec_elem| vec_elem.get_name() == name)
    }
    pub fn get_by_name_mut(&mut self, name: &str) -> Option<&mut DataCell> {
        self.0
            .iter_mut()
            .find(|vec_elem| vec_elem.get_name() == name)
    }

    pub fn del_by_idx(&mut self, idx: usize) -> Result<DataCell> {
        let idx = self
            .0
            .iter()
            .position(|vec_elem| vec_elem.get_idx() == idx)
            .ok_or(VenumTdsError::DataAccess(
                DataAccessErrors::IllegalIdxAccess { idx }, // TODO: better error...
            ))?;
        Ok(self.0.swap_remove(idx))
    }

    pub fn push(&mut self, elem: DataCell) {
        self.0.push(elem);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl IntoIterator for DataCellRow {
    type Item = DataCell;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct DataValueRow(pub Vec<Value>);

impl From<DataCellRow> for DataValueRow {
    fn from(mut vcr: DataCellRow) -> Self {
        // TODO: this is not really ...correct, depending on the definition.
        //       we should probably insert the entries into the plain vector
        //       in the correct index order from the source DataCellRow.
        Self(
            vcr.0
                .iter_mut()
                .map(|v| std::mem::take(&mut v.data))
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use venum::venum::{Value, ValueType};

    use crate::{data_cell::DataCell, data_cell_row::DataCellRow};

    use super::DataValueRow;

    #[test]
    pub fn test_try_from_w_data() {
        let mut c = DataCellRow::new();

        let vc1 = DataCell::new(
            ValueType::String,
            String::from("foo"),
            0,
            Value::String(String::from("meh")),
        );

        let vc2 = DataCell::new(
            ValueType::String,
            String::from("bar"),
            1,
            Value::String(String::from("meh2")),
        );

        c.0.push(vc1);
        c.0.push(vc2);

        let mut res: DataValueRow = c.into();
        assert_eq!(
            String::from("meh2"),
            String::try_from(res.0.pop().unwrap()).unwrap()
        );
        assert_eq!(Value::String(String::from("meh")), res.0.pop().unwrap());
    }

    #[test]
    pub fn test_try_from_wo_data() {
        let mut c = DataCellRow::new();
        let vc1 = DataCell::new(ValueType::String, String::from("foo"), 0, Value::None);
        let vc2 = DataCell::new(ValueType::String, String::from("bar"), 1, Value::None);
        c.0.push(vc1);
        c.0.push(vc2);

        let mut res: DataValueRow = c.into();
        assert_eq!(Value::None, res.0.pop().unwrap());
        assert_eq!(Value::None, res.0.pop().unwrap());
    }

    #[test]
    pub fn test_try_from_w_mixed_data() {
        let mut c = DataCellRow::new();
        let vc1 = DataCell::new(ValueType::String, String::from("foo"), 0, Value::None);
        let vc2 = DataCell::new(ValueType::String, String::from("bar"), 1, Value::None);
        let vc3 = DataCell::new(
            ValueType::String,
            String::from("col3"),
            2,
            Value::String(String::from("baz")),
        );
        c.0.push(vc1);
        c.0.push(vc2);
        c.0.push(vc3);

        let mut res: DataValueRow = c.into();
        assert_eq!(Value::String(String::from("baz")), res.0.pop().unwrap());
        assert_eq!(Value::None, res.0.pop().unwrap());
        assert_eq!(Value::None, res.0.pop().unwrap());
    }

    #[test]
    pub fn test_index_access() {
        let mut c = DataCellRow::new();

        let vc1 = DataCell::new(
            ValueType::String,
            String::from("foo"),
            123,
            Value::String(String::from("meh")),
        );
        c.0.push(vc1);

        let res = c.get_by_idx(123).unwrap();
        assert_eq!(123, res.idx);
    }

    #[test]
    pub fn test_named_access() {
        let mut c = DataCellRow::new();

        let vc1 = DataCell::new(
            ValueType::String,
            String::from("foo"),
            123,
            Value::String(String::from("meh")),
        );
        c.0.push(vc1);

        let res = c.get_by_name("foo").unwrap();
        assert_eq!("foo", res.name);
    }
}
