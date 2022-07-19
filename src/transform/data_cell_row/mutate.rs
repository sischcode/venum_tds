use crate::{
    data_cell::DataCell,
    data_cell_row::DataCellRow,
    errors::{ContainerOpsErrors, Result, VenumTdsError},
    transform::data_cell::splitting::SplitDataCell,
};

pub trait TransrichDataCellRowInplace {
    fn transrich(&self, data_cell_row: &mut DataCellRow) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MutateItemIdx {
    pub from: usize,
    pub to: usize,
}
impl MutateItemIdx {
    pub fn new(from: usize, to: usize) -> Self {
        Self { from, to }
    }
}
impl TransrichDataCellRowInplace for MutateItemIdx {
    fn transrich(&self, data_cell_row: &mut DataCellRow) -> Result<()> {
        let container_entry = data_cell_row.get_by_idx_mut(self.from);
        match container_entry {
            None => Err(VenumTdsError::ContainerOps(ContainerOpsErrors::Generic {
                msg: String::from("No DataEntry with idx {self.from}. Can't mutate index."),
            })),
            Some(date_entry) => {
                date_entry.set_idx(self.to);
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeleteItemAtIdx(pub usize);
impl TransrichDataCellRowInplace for DeleteItemAtIdx {
    fn transrich(&self, data_cell_row: &mut DataCellRow) -> Result<()> {
        data_cell_row.del_by_idx(self.0).map(|_| ())
    }
}

pub struct AddItem(pub DataCell);
impl TransrichDataCellRowInplace for AddItem {
    fn transrich(&self, data_cell_row: &mut DataCellRow) -> Result<()> {
        data_cell_row.push(self.0.clone());
        Ok(())
    }
}

pub struct SplitItemAtIdx<S: SplitDataCell> {
    pub idx: usize,
    pub splitter: S,
    pub delete_source_item: bool,
}

impl<S> TransrichDataCellRowInplace for SplitItemAtIdx<S>
where
    S: SplitDataCell,
{
    fn transrich(&self, data_cell_row: &mut DataCellRow) -> Result<()> {
        let entry = data_cell_row.get_by_idx_mut(self.idx).ok_or_else(|| {
            VenumTdsError::ContainerOps(ContainerOpsErrors::SplitItemError {
                idx: self.idx,
                msg: format!("Container does not have an entry at idx: {}", self.idx),
            })
        })?;

        let (left, right) = self.splitter.split(entry)?;

        data_cell_row.push(left);
        data_cell_row.push(right);
        if self.delete_source_item {
            data_cell_row.del_by_idx(self.idx).unwrap();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use venum::venum::{Value, ValueType};

    use crate::transform::{
        data_cell::splitting::SplitDataCellUsingValueSplit, data_cell_row::mutate::*,
        value::spliting::ValueStringSeparatorCharSplit,
    };

    #[test]
    fn test_mutate_idx_of_tds_data_cell() {
        let m = MutateItemIdx::new(0, 1);

        let mut c = DataCellRow::new();
        c.0.push(DataCell::new_without_data(
            ValueType::Bool,
            String::from("col1"),
            0,
        ));

        m.transrich(&mut c).unwrap();
        assert_eq!(1, c.0.first().unwrap().idx);
    }

    #[test]
    fn test_delete_from_container() {
        let mut c = DataCellRow::new();
        c.0.push(DataCell::new_without_data(
            ValueType::Bool,
            String::from("col1"),
            0,
        ));
        c.0.push(DataCell::new_without_data(
            ValueType::Bool,
            String::from("col2"),
            1,
        ));

        let container_transricher = DeleteItemAtIdx(0);
        container_transricher.transrich(&mut c).unwrap();

        let container_transricher2 = DeleteItemAtIdx(1);
        container_transricher2.transrich(&mut c).unwrap();

        assert_eq!(0, c.0.len());
    }

    #[test]
    #[should_panic(expected = "DataAccess(IllegalIdxAccess { idx: 0 })")]
    fn test_delete_from_container_err() {
        let mut c = DataCellRow::new();
        let container_transricher = DeleteItemAtIdx(0);
        container_transricher.transrich(&mut c).unwrap();
    }

    #[test]
    fn test_add_to_container() {
        let mut c = DataCellRow::new();
        let container_transricher = AddItem(DataCell::new_without_data(
            ValueType::Bool,
            String::from("col1"),
            0,
        ));
        container_transricher.transrich(&mut c).unwrap();

        assert_eq!(1, c.0.len());
    }

    #[test]
    fn test_combined() {
        let mut c = DataCellRow::new();
        c.0.push(DataCell::new_without_data(
            ValueType::Bool,
            String::from("col1"),
            0,
        ));
        c.0.push(DataCell::new_without_data(
            ValueType::Bool,
            String::from("col2"),
            1,
        ));

        let mut transrichers: Vec<Box<dyn TransrichDataCellRowInplace>> = Vec::with_capacity(4);

        transrichers.push(Box::new(AddItem(DataCell::new_without_data(
            ValueType::Bool,
            String::from("col3"),
            2,
        ))));
        transrichers.push(Box::new(DeleteItemAtIdx(1)));
        transrichers.push(Box::new(DeleteItemAtIdx(2)));
        transrichers.push(Box::new(MutateItemIdx::new(0, 10)));

        transrichers
            .iter_mut()
            .map(|t| t.transrich(&mut c))
            .collect::<Result<Vec<()>>>()
            .unwrap();

        assert_eq!(1, c.0.len());
        assert_eq!(10, c.0.first().unwrap().get_idx());
        assert_eq!(String::from("col1"), c.0.first().unwrap().get_name());
    }

    #[test]
    pub fn test_split_container_item_using_value_string_separator_char_divider() {
        let mut c = DataCellRow::new();
        c.0.push(DataCell::new(
            ValueType::String,
            String::from("col1"),
            0,
            Value::String(String::from("32.3:1")),
        ));

        let data_cell_splitter = SplitDataCellUsingValueSplit {
            splitter: ValueStringSeparatorCharSplit {
                sep_char: ':',
                split_none: false,
            },
            target_left: DataCell::new_without_data(ValueType::Float32, String::from("col2"), 1),
            target_right: DataCell::new_without_data(ValueType::Int8, String::from("col3"), 2),
        };

        let split_item_at_idx = SplitItemAtIdx {
            idx: 0,
            splitter: data_cell_splitter,
            delete_source_item: false,
        };

        split_item_at_idx.transrich(&mut c).unwrap();

        assert_eq!(3, c.0.len());
        assert_eq!(&Value::Float32(32.3), c.get_by_idx(1).unwrap().get_data());
        assert_eq!(&Value::Int8(1_i8), c.get_by_idx(2).unwrap().get_data());
    }

    #[test]
    pub fn test_split_container_item_using_value_string_separator_char_divider_delete_src() {
        let mut c = DataCellRow::new();
        c.0.push(DataCell::new(
            ValueType::String,
            String::from("col1"),
            0,
            Value::String(String::from("32.3:1")),
        ));

        let data_cell_splitter = SplitDataCellUsingValueSplit {
            splitter: ValueStringSeparatorCharSplit {
                sep_char: ':',
                split_none: false,
            },
            target_left: DataCell::new_without_data(ValueType::Float32, String::from("col2"), 1),
            target_right: DataCell::new_without_data(ValueType::Int8, String::from("col3"), 2),
        };

        let div_at = SplitItemAtIdx {
            idx: 0,
            splitter: data_cell_splitter,
            delete_source_item: true,
        };

        div_at.transrich(&mut c).unwrap();

        assert_eq!(2, c.0.len()); // <-- we deleted the original
        assert_eq!(&Value::Float32(32.3), c.get_by_idx(1).unwrap().get_data());
        assert_eq!(&Value::Int8(1_i8), c.get_by_idx(2).unwrap().get_data());
    }

    #[test]
    pub fn test_split_container_item_using_value_string_separator_char_divider_none() {
        let mut c = DataCellRow::new();
        c.0.push(DataCell::new_without_data(
            ValueType::String,
            String::from("col1"),
            0,
        ));

        let data_cell_splitter = SplitDataCellUsingValueSplit {
            splitter: ValueStringSeparatorCharSplit {
                sep_char: ':',
                split_none: true,
            },
            target_left: DataCell::new_without_data(ValueType::Float32, String::from("col2"), 1),
            target_right: DataCell::new_without_data(ValueType::Int8, String::from("col3"), 2),
        };

        let split_item_at_idx = SplitItemAtIdx {
            idx: 0,
            splitter: data_cell_splitter,
            delete_source_item: false,
        };

        split_item_at_idx.transrich(&mut c).unwrap();

        assert_eq!(3, c.0.len());
        assert_eq!(&Value::None, c.get_by_idx(1).unwrap().get_data());
        assert_eq!(&Value::None, c.get_by_idx(2).unwrap().get_data());
    }

    #[test]
    pub fn test_split_container_item_using_value_string_separator_char_divider_none_delete_src() {
        let mut c = DataCellRow::new();
        c.0.push(DataCell::new_without_data(
            ValueType::String,
            String::from("col1"),
            0,
        ));

        let data_cell_splitter = SplitDataCellUsingValueSplit {
            splitter: ValueStringSeparatorCharSplit {
                sep_char: ':',
                split_none: true,
            },
            target_left: DataCell::new_without_data(ValueType::Float32, String::from("col2"), 1),
            target_right: DataCell::new_without_data(ValueType::Int8, String::from("col3"), 2),
        };

        let split_item_at_idx = SplitItemAtIdx {
            idx: 0,
            splitter: data_cell_splitter,
            delete_source_item: true,
        };

        split_item_at_idx.transrich(&mut c).unwrap();

        assert_eq!(2, c.0.len()); // <--- !!!
        assert_eq!(&Value::None, c.get_by_idx(1).unwrap().get_data());
        assert_eq!(&Value::None, c.get_by_idx(2).unwrap().get_data());
    }

    #[test]
    #[should_panic(
        expected = "Split(SplitError { msg: \"Value is None, but split_none is false\", src_val: None, details: None })"
    )]
    pub fn test_split_container_item_using_value_string_separator_char_divider_none_but_split_none_is_false(
    ) {
        let mut c = DataCellRow::new();
        c.0.push(DataCell::new_without_data(
            ValueType::String,
            String::from("col1"),
            0,
        ));

        let data_cell_splitter = SplitDataCellUsingValueSplit {
            splitter: ValueStringSeparatorCharSplit {
                sep_char: ':',
                split_none: false,
            },
            target_left: DataCell::new_without_data(ValueType::Float32, String::from("col2"), 1),
            target_right: DataCell::new_without_data(ValueType::Int8, String::from("col3"), 2),
        };

        let split_item_at_idx = SplitItemAtIdx {
            idx: 0,
            splitter: data_cell_splitter,
            delete_source_item: true,
        };

        split_item_at_idx.transrich(&mut c).unwrap();
    }
}
