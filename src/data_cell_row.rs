use crate::errors::{DataAccessErrors, Result, VenumTdsError};

use super::data_cell::DataCell;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct DataCellRow(pub Vec<DataCell>); // TODO: we actually don't want this to be public, but we still have code in patti_csv that relies on it.

impl DataCellRow {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
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
                DataAccessErrors::IllegalIdxAccess { idx },
            ))?;
        Ok(self.0.swap_remove(idx))
    }

    pub fn push(&mut self, elem: DataCell) {
        self.0.push(elem);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl IntoIterator for DataCellRow {
    type Item = DataCell;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use venum::value::Value;
    use venum::value_type::ValueType;

    use crate::{data_cell::DataCell, data_cell_row::DataCellRow};

    #[test]
    pub fn index_access() {
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
    pub fn named_access() {
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
