use crate::{data_cell::DataCell, errors::Result};

pub trait MergeDataCell {
    fn merge(item_1: &DataCell, item_2: &DataCell) -> Result<DataCell>;
}

pub trait MergeDataCellN {
    fn merge(items: Vec<&DataCell>) -> Result<DataCell>;
}
