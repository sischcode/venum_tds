use std::fmt::Debug;

use crate::{data_cell_row::DataCellRow, errors::Result};

use super::mutate::TransrichDataCellRowInplace;

#[derive(Debug)]
pub struct TransrichPass {
    pub transformer: Vec<Box<dyn TransrichDataCellRowInplace>>,
    pub order: Option<Vec<Box<dyn TransrichDataCellRowInplace>>>,
}

impl TransrichPass {
    pub fn transrich(&self, container: &mut DataCellRow) -> Result<()> {
        self.transformer
            .iter()
            .try_for_each(|tri| tri.transrich(container))?;

        if let Some(orderings) = &self.order {
            orderings.iter().try_for_each(|o| o.transrich(container))?;
        }
        Ok(())
    }
}

pub struct TransrichPassesConfig {
    pub passes: Vec<TransrichPass>,
}

impl TransrichPassesConfig {
    pub fn transrich(&self, container: &mut DataCellRow) -> Result<()> {
        self.passes
            .iter()
            .try_for_each(|pass| pass.transrich(container))
    }
}

#[cfg(test)]
mod tests {

    // TODO: more tests!

    use venum::venum::{Value, ValueType};

    use crate::{
        data_cell::DataCell,
        data_cell_row::DataCellRow,
        transform::{
            data_cell::splitting::SplitDataCellUsingValueSplit,
            data_cell_row::{
                mutate::*,
                transrich_pass::{TransrichPass, TransrichPassesConfig},
            },
            value::spliting::ValueStringSeparatorCharSplit,
        },
    };

    #[test]
    fn test_transrich_pass_del_after_split() {
        let trp: TransrichPass = TransrichPass {
            transformer: vec![Box::new(SplitItemAtIdx {
                delete_source_item: true,
                idx: 0,
                splitter: SplitDataCellUsingValueSplit {
                    splitter: ValueStringSeparatorCharSplit {
                        sep_char: ' ',
                        split_none: true,
                    },
                    target_left: DataCell::new_without_data(
                        ValueType::Float32,
                        String::from("amount"),
                        1,
                    ),
                    target_right: DataCell::new_without_data(
                        ValueType::String,
                        String::from("currency"),
                        2,
                    ),
                },
            })],
            order: Some(vec![
                Box::new(MutateItemIdx { from: 1, to: 0 }), // CAUTION!!!
                Box::new(MutateItemIdx { from: 2, to: 1 }), // You need to order from low to high!
            ]),
        };

        let mut data = DataCellRow::new();
        data.push(DataCell::new(
            ValueType::String,
            String::from("amount+currency"),
            0,
            Value::String(String::from("10.10 CHF")),
        ));

        trp.transrich(&mut data).unwrap();

        assert_eq!(2, data.len());
        assert_eq!(Value::Float32(10.10), data.get_by_idx(0).unwrap().data);
        assert_eq!(
            Value::String(String::from("CHF")),
            data.get_by_idx(1).unwrap().data
        );
    }

    #[test]
    fn test_transrich_pass_remain_after_split_then_delete() {
        let trp: TransrichPass = TransrichPass {
            transformer: vec![
                Box::new(SplitItemAtIdx {
                    delete_source_item: false,
                    idx: 0,
                    splitter: SplitDataCellUsingValueSplit {
                        splitter: ValueStringSeparatorCharSplit {
                            sep_char: ' ',
                            split_none: true,
                        },
                        target_left: DataCell::new_without_data(
                            ValueType::Float32,
                            String::from("amount"),
                            1,
                        ),
                        target_right: DataCell::new_without_data(
                            ValueType::String,
                            String::from("currency"),
                            2,
                        ),
                    },
                }),
                Box::new(DeleteItemAtIdx { 0: 0 }),
            ],
            order: Some(vec![
                Box::new(MutateItemIdx { from: 1, to: 0 }), // CAUTION!!!
                Box::new(MutateItemIdx { from: 2, to: 1 }), // You need to order from low to high!
            ]),
        };

        let mut data = DataCellRow::new();
        data.push(DataCell::new(
            ValueType::String,
            String::from("amount+currency"),
            0,
            Value::String(String::from("10.10 CHF")),
        ));

        trp.transrich(&mut data).unwrap();

        assert_eq!(2, data.len());
        assert_eq!(Value::Float32(10.10), data.get_by_idx(0).unwrap().data);
        assert_eq!(
            Value::String(String::from("CHF")),
            data.get_by_idx(1).unwrap().data
        );
    }

    #[test]
    fn test_transrich_passes() {
        let trp1: TransrichPass = TransrichPass {
            transformer: vec![Box::new(SplitItemAtIdx {
                delete_source_item: false, // <--- !!!
                idx: 0,
                splitter: SplitDataCellUsingValueSplit {
                    splitter: ValueStringSeparatorCharSplit {
                        sep_char: ' ',
                        split_none: true,
                    },
                    target_left: DataCell::new_without_data(
                        ValueType::Float32,
                        String::from("amount"),
                        1,
                    ),
                    target_right: DataCell::new_without_data(
                        ValueType::String,
                        String::from("currency"),
                        2,
                    ),
                },
            })],
            order: Some(vec![
                Box::new(MutateItemIdx { from: 0, to: 3 }), // move the old "column" out of the way
                Box::new(MutateItemIdx { from: 1, to: 0 }),
                Box::new(MutateItemIdx { from: 2, to: 1 }),
            ]),
        };

        let trp2: TransrichPass = TransrichPass {
            transformer: vec![Box::new(DeleteItemAtIdx { 0: 3 })],
            order: None,
        };

        let mut data = DataCellRow::new();
        data.push(DataCell::new(
            ValueType::String,
            String::from("amount+currency"),
            0,
            Value::String(String::from("10.10 CHF")),
        ));

        let passes_config = TransrichPassesConfig {
            passes: vec![trp1, trp2],
        };

        passes_config.transrich(&mut data).unwrap();

        assert_eq!(2, data.len());
        assert_eq!(Value::Float32(10.10), data.get_by_idx(0).unwrap().data);
        assert_eq!(
            Value::String(String::from("CHF")),
            data.get_by_idx(1).unwrap().data
        );
    }
}
