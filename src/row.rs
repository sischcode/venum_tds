use venum::venum::Value;

use crate::{
    errors::{DataAccessErrors, Result, VenumTdsError},
    traits::{DataContainer, DataEntry},
};

use super::cell::DataCell;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct DataCellRow(pub Vec<DataCell>);

impl DataCellRow {
    pub fn new() -> Self {
        Self { 0: Vec::new() }
    }
}

impl DataContainer<DataCell> for DataCellRow {
    fn get_by_idx(&self, idx: usize) -> Option<&DataCell> {
        self.0.iter().find(|&vec_elem| vec_elem.get_idx() == idx)
    }
    fn get_by_idx_mut(&mut self, idx: usize) -> Option<&mut DataCell> {
        self.0.iter_mut().find(|vec_elem| vec_elem.get_idx() == idx)
    }

    fn get_by_name(&self, name: &str) -> Option<&DataCell> {
        self.0.iter().find(|&vec_elem| vec_elem.get_name() == name)
    }
    fn get_by_name_mut(&mut self, name: &str) -> Option<&mut DataCell> {
        self.0
            .iter_mut()
            .find(|vec_elem| vec_elem.get_name() == name)
    }

    fn del_by_idx(&mut self, idx: usize) -> Result<DataCell> {
        let idx = self
            .0
            .iter()
            .position(|vec_elem| vec_elem.get_idx() == idx)
            .ok_or(VenumTdsError::DataAccess(
                DataAccessErrors::IllegalIdxAccess { idx }, // TODO: better error...
            ))?;
        Ok(self.0.swap_remove(idx))
    }

    fn add(&mut self, elem: DataCell) {
        self.0.push(elem);
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
pub struct DataValueRow(pub Vec<Option<Value>>);

impl From<DataCellRow> for DataValueRow {
    fn from(mut vcr: DataCellRow) -> Self {
        // TODO: this is not really ...correct, depending on the definition.
        //       we should probably insert the entries into the plain vector
        //       in the correct index order from the source DataCellRow.
        Self {
            0: vcr
                .0
                .iter_mut()
                .map(|v| std::mem::take(&mut v.data))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use venum::venum::Value;

    use crate::{cell::DataCell, row::DataCellRow, traits::DataContainer};

    use super::DataValueRow;

    #[test]
    pub fn test_try_from_w_data() {
        let mut c = DataCellRow::new();

        let vc1 = DataCell::new(
            Value::string_default(),
            String::from("foo"),
            0,
            Some(Value::String(String::from("meh"))),
        );

        let vc2 = DataCell::new(
            Value::string_default(),
            String::from("bar"),
            1,
            Some(Value::String(String::from("meh2"))),
        );

        c.0.push(vc1);
        c.0.push(vc2);

        let r: DataValueRow = c.into();
        println!("{:?}", r);
    }

    #[test]
    pub fn test_try_from_wo_data() {
        let mut c = DataCellRow::new();
        let vc1 = DataCell::new_without_data(Value::string_default(), String::from("foo"), 0);
        let vc2 = DataCell::new_without_data(Value::string_default(), String::from("bar"), 1);
        c.0.push(vc1);
        c.0.push(vc2);

        let r: DataValueRow = c.into();
        println!("{:?}", r);
    }

    #[test]
    pub fn test_try_from_w_mixed_data_1() {
        // to be clear. The constructed case here can't (well, shouldn't) happen in our use case, since we
        // always parse a complete line. The real world case is more like in: test_try_from_w_mixed_data_2
        let mut c = DataCellRow::new();
        let vc1 = DataCell::new_without_data(Value::string_default(), String::from("foo"), 0);
        let vc2 = DataCell::new_without_data(Value::string_default(), String::from("bar"), 1);
        let vc3 = DataCell::new(
            Value::string_default(),
            String::from("col3"),
            3,
            Some(Value::String(String::from("baz"))),
        );
        c.0.push(vc1);
        c.0.push(vc2);
        c.0.push(vc3);

        let r: DataValueRow = c.into();
        println!("{:?}", r);
    }

    #[test]
    pub fn test_try_from_w_mixed_data_2() {
        let mut c = DataCellRow::new();
        let vc1 = DataCell::new(Value::string_default(), String::from("foo"), 0, None);
        let vc2 = DataCell::new(Value::string_default(), String::from("bar"), 1, None);
        let vc3 = DataCell::new(
            Value::string_default(),
            String::from("col3"),
            3,
            Some(Value::String(String::from("baz"))),
        );
        c.0.push(vc1);
        c.0.push(vc2);
        c.0.push(vc3);

        let r: DataValueRow = c.into();
        println!("{:?}", r);
    }

    #[test]
    pub fn test_index_access() {
        let mut c = DataCellRow::new();

        let vc1 = DataCell::new(
            Value::string_default(),
            String::from("foo"),
            123,
            Some(Value::String(String::from("meh"))),
        );
        c.0.push(vc1);

        let res = c.get_by_idx(123).unwrap();
        assert_eq!(123, res.idx);
    }

    #[test]
    pub fn test_named_access() {
        let mut c = DataCellRow::new();

        let vc1 = DataCell::new(
            Value::string_default(),
            String::from("foo"),
            123,
            Some(Value::String(String::from("meh"))),
        );
        c.0.push(vc1);

        let res = c.get_by_name("foo").unwrap();
        assert_eq!("foo", res.name);
    }
}
