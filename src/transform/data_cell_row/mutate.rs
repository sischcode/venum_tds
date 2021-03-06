use std::fmt::Debug;

use venum::venum::{Value, ValueType};

use crate::{
    data_cell::DataCell,
    data_cell_row::DataCellRow,
    errors::{ContainerOpsErrors, Result, VenumTdsError},
    transform::{
        data_cell::splitting::SplitDataCell,
        util::chrono_utils::utc_datetime_as_fixed_offset_datetime,
    },
};

pub trait TransrichDataCellRowInplace: Debug {
    fn transrich(&self, data_cell_row: &mut DataCellRow) -> Result<()>;
}

pub trait TransrichDataCellRowInplaceStateful: Debug {
    fn transrich(&mut self, data_cell_row: &mut DataCellRow) -> Result<()>;
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

#[derive(Debug, Clone, PartialEq)]
pub struct AddItemStatic(pub DataCell);
impl TransrichDataCellRowInplace for AddItemStatic {
    fn transrich(&self, data_cell_row: &mut DataCellRow) -> Result<()> {
        data_cell_row.push(self.0.clone());
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AddItemMetadata(pub DataCell);
impl TransrichDataCellRowInplace for AddItemMetadata {
    fn transrich(&self, data_cell_row: &mut DataCellRow) -> Result<()> {
        data_cell_row.push(self.0.clone());
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "jsonconf", derive(serde::Deserialize))]
pub enum RuntimeValue {
    CurrentDateTimeUtcAsFixedOffset,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "jsonconf", derive(serde::Deserialize))]
pub enum RuntimeValueStateful {
    RowEnumeration,
}

#[derive(Debug, PartialEq)]
pub struct AddItemRuntimeStatefulRowEnum {
    num_invoke: Option<u128>,
    pub header: Option<String>,
    pub idx: usize,
}
impl TransrichDataCellRowInplaceStateful for AddItemRuntimeStatefulRowEnum {
    fn transrich(&mut self, data_cell_row: &mut DataCellRow) -> Result<()> {
        if self.num_invoke.is_none() {
            self.num_invoke = Some(1);
        } else {
            self.num_invoke = self.num_invoke.map(|old| old + 1);
        }
        let curr_enum_cell = DataCell::new(
            ValueType::UInt128,
            self.header.clone().unwrap_or_else(|| self.idx.to_string()),
            self.idx,
            Value::UInt128(self.num_invoke.unwrap()), // we set it right above!
        );
        data_cell_row.push(curr_enum_cell);

        Ok(())
    }
}
impl AddItemRuntimeStatefulRowEnum {
    pub fn new(header: Option<String>, idx: usize) -> Self {
        Self {
            num_invoke: None,
            header: header,
            idx: idx,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AddItemRuntime {
    pub header: Option<String>,
    pub idx: usize,
    pub rtv: RuntimeValue,
}
impl TransrichDataCellRowInplace for AddItemRuntime {
    fn transrich(&self, data_cell_row: &mut DataCellRow) -> Result<()> {
        match self.rtv {
            RuntimeValue::CurrentDateTimeUtcAsFixedOffset => {
                let curr_date_cell = DataCell::new(
                    ValueType::DateTime,
                    self.header.clone().unwrap_or_else(|| self.idx.to_string()),
                    self.idx,
                    Value::DateTime(utc_datetime_as_fixed_offset_datetime(
                        chrono::offset::Utc::now(),
                    )),
                );
                data_cell_row.push(curr_date_cell);
                Ok(())
            }
            // _ => Err(VenumTdsError::ContainerOps(ContainerOpsErrors::Generic {
            //     msg: format!("{:?} not implemented. (idx={}", &self.rtv, &self.idx),
            // })),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AddItemRuntimeSingleton(DataCell);
impl AddItemRuntimeSingleton {
    pub fn new(header: Option<String>, idx: usize, rtv: RuntimeValue) -> Result<Self> {
        match rtv {
            RuntimeValue::CurrentDateTimeUtcAsFixedOffset => {
                Ok(AddItemRuntimeSingleton(DataCell::new(
                    venum::venum::ValueType::DateTime,
                    header.unwrap_or_else(|| idx.to_string()),
                    idx,
                    Value::DateTime(utc_datetime_as_fixed_offset_datetime(
                        chrono::offset::Utc::now(),
                    )),
                )))
            }
            // _ => Err(VenumTdsError::ContainerOps(ContainerOpsErrors::Generic {
            //     msg: format!("{:?} not implemented. (idx={}", &rtv, &idx),
            // })),
        }
    }
}

impl TransrichDataCellRowInplace for AddItemRuntimeSingleton {
    fn transrich(&self, data_cell_row: &mut DataCellRow) -> Result<()> {
        data_cell_row.push(self.0.clone());
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
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
            data_cell_row.del_by_idx(self.idx).unwrap(); // we check it above already
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
    fn mutate_idx_of_tds_data_cell() {
        let m = MutateItemIdx::new(0, 1);

        let mut c = DataCellRow::new();
        c.push(DataCell::new_without_data(
            ValueType::Bool,
            String::from("col1"),
            0,
        ));

        m.transrich(&mut c).unwrap();
        assert_eq!(1, c.get_by_idx(1).unwrap().idx);
    }

    #[test]
    fn delete_from_container() {
        let mut c = DataCellRow::new();
        c.push(DataCell::new_without_data(
            ValueType::Bool,
            String::from("col1"),
            0,
        ));
        c.push(DataCell::new_without_data(
            ValueType::Bool,
            String::from("col2"),
            1,
        ));

        let container_transricher = DeleteItemAtIdx(0);
        container_transricher.transrich(&mut c).unwrap();

        let container_transricher2 = DeleteItemAtIdx(1);
        container_transricher2.transrich(&mut c).unwrap();

        assert_eq!(0, c.len());
    }

    #[test]
    #[should_panic(expected = "DataAccess(IllegalIdxAccess { idx: 0 })")]
    fn delete_from_container_err() {
        let mut c = DataCellRow::new();
        let container_transricher = DeleteItemAtIdx(0);
        container_transricher.transrich(&mut c).unwrap();
    }

    #[test]
    fn add_item_static() {
        let mut c = DataCellRow::new();
        let container_transricher = AddItemStatic(DataCell::new_without_data(
            ValueType::Bool,
            String::from("col1"),
            0,
        ));
        container_transricher.transrich(&mut c).unwrap();

        assert_eq!(1, c.len());
    }

    #[test]
    fn add_item_runtime() {
        let mut c1 = DataCellRow::new();
        let container_transricher = AddItemRuntime {
            header: Some(String::from("col1")),
            idx: 0,
            rtv: RuntimeValue::CurrentDateTimeUtcAsFixedOffset,
        };
        container_transricher.transrich(&mut c1).unwrap();
        assert_eq!(1, c1.len());
        // println!("{:?}", c1.get_by_idx(0).unwrap().get_data());

        let mut c2 = DataCellRow::new();
        container_transricher.transrich(&mut c2).unwrap();
        assert_eq!(1, c2.len());
        // println!("{:?}", c2.get_by_idx(0).unwrap().get_data());

        // In essence, that these are not the same (intentionally!), means,
        // that the DateTime Value is "lazily" constructed, whenever "transrich"
        // is called internally
        assert_ne!(
            c1.get_by_idx(0).unwrap().get_data(),
            c2.get_by_idx(0).unwrap().get_data()
        );
    }

    #[test]
    fn add_item_runtime_singleton() {
        let mut c1 = DataCellRow::new();
        let container_transricher = AddItemRuntimeSingleton::new(
            Some(String::from("col1")),
            0,
            RuntimeValue::CurrentDateTimeUtcAsFixedOffset,
        )
        .unwrap();

        container_transricher.transrich(&mut c1).unwrap();
        assert_eq!(1, c1.len());
        // println!("{:?}", c1.get_by_idx(0).unwrap().get_data());

        let mut c2 = DataCellRow::new();
        container_transricher.transrich(&mut c2).unwrap();
        assert_eq!(1, c2.len());
        // println!("{:?}", c2.get_by_idx(0).unwrap().get_data());

        // These should be the same, since we build the DateTime in the
        // new() and then store it.
        assert_eq!(
            c1.get_by_idx(0).unwrap().get_data(),
            c2.get_by_idx(0).unwrap().get_data()
        );
    }

    #[test]
    fn add_item_runtime_stateful_rownum() {
        let mut c1 = DataCellRow::new();

        let mut container_transricher =
            AddItemRuntimeStatefulRowEnum::new(Some(String::from("col1")), 0);

        container_transricher.transrich(&mut c1).unwrap();
        assert_eq!(1, c1.len());
        // println!("{:?}", &c1);

        let mut c2 = DataCellRow::new();
        container_transricher.transrich(&mut c2).unwrap();
        assert_eq!(1, c2.len());
        // println!("{:?}", &c2);

        assert_eq!(&Value::UInt128(2), c2.get_by_idx(0).unwrap().get_data());
    }

    #[test]
    fn combined() {
        let mut c = DataCellRow::new();
        c.push(DataCell::new_without_data(
            ValueType::Bool,
            String::from("col1"),
            0,
        ));
        c.push(DataCell::new_without_data(
            ValueType::Bool,
            String::from("col2"),
            1,
        ));

        let mut transrichers: Vec<Box<dyn TransrichDataCellRowInplace>> = Vec::with_capacity(4);

        transrichers.push(Box::new(AddItemStatic(DataCell::new_without_data(
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

        assert_eq!(1, c.len());
        assert_eq!(10, c.get_by_idx(10).unwrap().get_idx());
        assert_eq!(
            String::from("col1"),
            c.get_by_name("col1").unwrap().get_name()
        );
    }

    #[test]
    pub fn split_container_item_using_value_string_separator_char_divider() {
        let mut c = DataCellRow::new();
        c.push(DataCell::new(
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

        assert_eq!(3, c.len());
        assert_eq!(&Value::Float32(32.3), c.get_by_idx(1).unwrap().get_data());
        assert_eq!(&Value::Int8(1_i8), c.get_by_idx(2).unwrap().get_data());
    }

    #[test]
    pub fn split_container_item_using_value_string_separator_char_divider_delete_src() {
        let mut c = DataCellRow::new();
        c.push(DataCell::new(
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

        assert_eq!(2, c.len()); // <-- we deleted the original
        assert_eq!(&Value::Float32(32.3), c.get_by_idx(1).unwrap().get_data());
        assert_eq!(&Value::Int8(1_i8), c.get_by_idx(2).unwrap().get_data());
    }

    #[test]
    pub fn split_container_item_using_value_string_separator_char_divider_none() {
        let mut c = DataCellRow::new();
        c.push(DataCell::new_without_data(
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

        assert_eq!(3, c.len());
        assert_eq!(&Value::None, c.get_by_idx(1).unwrap().get_data());
        assert_eq!(&Value::None, c.get_by_idx(2).unwrap().get_data());
    }

    #[test]
    pub fn split_container_item_using_value_string_separator_char_divider_none_delete_src() {
        let mut c = DataCellRow::new();
        c.push(DataCell::new_without_data(
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

        assert_eq!(2, c.len()); // <--- !!!
        assert_eq!(&Value::None, c.get_by_idx(1).unwrap().get_data());
        assert_eq!(&Value::None, c.get_by_idx(2).unwrap().get_data());
    }

    #[test]
    #[should_panic(
        expected = "Split(SplitError { msg: \"Value is None, but split_none is false\", src_val: None, details: None })"
    )]
    pub fn split_container_item_using_value_string_separator_char_divider_none_but_split_none_is_false(
    ) {
        let mut c = DataCellRow::new();
        c.push(DataCell::new_without_data(
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
