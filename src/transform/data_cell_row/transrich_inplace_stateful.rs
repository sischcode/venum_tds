use std::fmt::Debug;

use venum::value::Value;

use crate::{data_cell::DataCell, data_cell_row::DataCellRow, errors::Result};

/// We get all info we need from the (static) transrichment config, we need to maintain state though, meaning, this CANNOT be reused!
/// In fact, this is (and enriching metadata values at runtime, i.e. with info from something exteranl) the main reason, why we cannot
/// "store" a TransrichPass, but need to store the config we construct a TransrichPass from, and then, construct it, when needed.
/// (The only workaround would be maintaining state outside of the implementing structs below)
pub trait TransrichInplaceStateful: Debug {
    fn transrich(&mut self, data_cell_row: &mut DataCellRow) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "jsonconf", derive(serde::Deserialize))]
pub enum RuntimeValueStateful {
    RowEnumeration,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AddItemRuntimeStatefulRowEnum {
    num_invoke: Option<u128>,
    pub header: Option<String>,
    pub idx: usize,
}
impl TransrichInplaceStateful for AddItemRuntimeStatefulRowEnum {
    fn transrich(&mut self, data_cell_row: &mut DataCellRow) -> Result<()> {
        if self.num_invoke.is_none() {
            self.num_invoke = Some(1);
        } else {
            self.num_invoke = self.num_invoke.map(|old| old + 1);
        }
        let curr_enum_cell = DataCell::new(
            self.header.clone().unwrap_or_else(|| self.idx.to_string()),
            self.idx,
            Value::UInt128(self.num_invoke.unwrap()), // we set it right above!
        )?;
        data_cell_row.push(curr_enum_cell);

        Ok(())
    }
}
impl AddItemRuntimeStatefulRowEnum {
    pub fn new(header: Option<String>, idx: usize) -> Self {
        Self {
            num_invoke: None,
            header,
            idx,
        }
    }
}

#[cfg(test)]
mod tests {
    use venum::value::Value;

    use crate::data_cell_row::DataCellRow;
    use crate::transform::data_cell_row::transrich_inplace_stateful::{
        AddItemRuntimeStatefulRowEnum, TransrichInplaceStateful,
    };

    #[test]
    fn add_item_runtime_stateful_rownum() {
        let mut c1 = DataCellRow::new();

        let mut container_transricher =
            AddItemRuntimeStatefulRowEnum::new(Some(String::from("col1")), 0);

        container_transricher.transrich(&mut c1).unwrap();
        assert_eq!(1, c1.len());

        let mut c2 = DataCellRow::new();
        container_transricher.transrich(&mut c2).unwrap();
        assert_eq!(1, c2.len());

        assert_eq!(&Value::UInt128(2), c2.get_by_idx(0).unwrap().get_data());
    }
}
