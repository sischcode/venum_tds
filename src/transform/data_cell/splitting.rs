use std::fmt::Debug;

use venum::value::Value;
use venum::value_type::ValueType;

use crate::data_cell::DataCell;
use crate::errors::{Result, SplitError, TransformErrors, VenumTdsError};
use crate::transform::value::spliting::ValueSplit;

pub trait SplitDataCell: Debug {
    fn split(&self, item: &DataCell) -> Result<(DataCell, DataCell)>;
}

fn converse_to(val: &Value, type_info: &ValueType) -> Result<Value> {
    match val {
        // we have the same enum variant in src and dst, we can use/clone it as is
        _ if std::mem::discriminant(val) == std::mem::discriminant(&Value::from(type_info)) => {
            Ok(val.clone())
        }
        Value::None => Ok(Value::None),
        // we have a String variant as src type try converting it to the target type
        Value::String(s) => Value::from_str_and_type(s, type_info).map_err(VenumTdsError::from),
        // TODO We can do better, but we don't support arbitrary convertions for now...
        _ => Err(VenumTdsError::Transform(TransformErrors::Split(
            SplitError::new(
                format!("type mismatch. {val:?} cannot be parsed/converted/put into destination of type {type_info:?}"),
                val.clone(),
            ),
        ))),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SplitDataCellUsingValueSplit<S: ValueSplit> {
    pub splitter: S,
    pub target_left: DataCell,
    pub target_right: DataCell,
}

impl<S> SplitDataCell for SplitDataCellUsingValueSplit<S>
where
    S: ValueSplit,
{
    fn split(&self, item: &DataCell) -> Result<(DataCell, DataCell)> {
        let (split_res_left, split_res_right) = self.splitter.split(item.get_data())?;

        let mut ctl = self.target_left.clone();
        if split_res_left.is_some() {
            ctl.set_data(converse_to(&split_res_left, ctl.get_type_info())?);
        } else {
            ctl.set_data(Value::None);
        }

        let mut ctr = self.target_right.clone();
        if split_res_left.is_some() {
            ctr.set_data(converse_to(&split_res_right, ctr.get_type_info())?);
        } else {
            ctr.set_data(Value::None);
        }

        Ok((ctl, ctr))
    }
}

#[cfg(test)]
mod tests {
    use venum::value::Value;
    use venum::value_type::ValueType;

    use crate::transform::value::spliting::{
        ValueStringRegexPairSplit, ValueStringSeparatorCharSplit,
    };

    use super::*;

    #[test]
    fn split_data_cell_using_value_split_sep_char_split() {
        let data = DataCell::new(
            ValueType::String,
            String::from("col1"),
            0,
            Value::String(String::from("foo 1")),
        );

        let split_using = SplitDataCellUsingValueSplit {
            splitter: ValueStringSeparatorCharSplit {
                sep_char: ' ',
                split_none: true,
            },
            target_left: DataCell::new_without_data(ValueType::String, String::from("col2"), 1),
            target_right: DataCell::new_without_data(ValueType::Int8, String::from("col3"), 2),
        };

        let (res_left, res_right) = split_using.split(&data).unwrap();

        assert_eq!(&Value::String(String::from("foo")), res_left.get_data());
        assert_eq!(&Value::Int8(1_i8), res_right.get_data());
    }

    #[test]
    fn split_data_cell_using_value_split_regex_split() {
        let data = DataCell::new(
            ValueType::String,
            String::from("col1"),
            0,
            Value::String(String::from("1.12 2.23")),
        );

        let split_using = SplitDataCellUsingValueSplit {
            splitter: ValueStringRegexPairSplit::new(
                "(\\d+\\.\\d+).*(\\d+\\.\\d+)".to_string(),
                true,
            )
            .unwrap(),
            target_left: DataCell::new_without_data(ValueType::Float32, String::from("col2"), 1),
            target_right: DataCell::new_without_data(ValueType::Float32, String::from("col3"), 2),
        };

        let (res_left, res_right) = split_using.split(&data).unwrap();

        assert_eq!(&Value::Float32(1.12), res_left.get_data());
        assert_eq!(&Value::Float32(2.23), res_right.get_data());
    }
}
