use venum::venum::Value;

use crate::{
    conf::jsonconf::{SplitterType, TransformEnrichPassConfig, TransformerConfig},
    data_cell::DataCell,
    errors::{Result, VenumTdsError},
    transform::{
        data_cell::splitting::SplitDataCellUsingValueSplit,
        data_cell_row::{
            mutate::{AddItemStatic, DeleteItemAtIdx, SplitItemAtIdx, TransrichDataCellRowInplace},
            transrich_pass::TransrichPass,
        },
        value::spliting::{ValueStringRegexPairSplit, ValueStringSeparatorCharSplit},
    },
};

use super::jsonconf::AddItemType;

impl TryFrom<TransformEnrichPassConfig> for TransrichPass {
    // TODO: (meta-hashmap, tep-config) instead of only tep-config
    type Error = VenumTdsError;

    fn try_from(tepc: TransformEnrichPassConfig) -> Result<Self> {
        let mut transrichers: Vec<Box<dyn TransrichDataCellRowInplace>> =
            Vec::with_capacity(tepc.transformers.len());
        for tc in tepc.transformers {
            match tc {
                TransformerConfig::DeleteItems { cfg } => {
                    for i in cfg {
                        transrichers.push(Box::new(DeleteItemAtIdx { 0: i }));
                    }
                }
                TransformerConfig::SplitItem { cfg } => {
                    let target_left = DataCell::new_without_data(
                        cfg.target_left.target_type,
                        cfg.target_left
                            .header
                            .unwrap_or_else(|| cfg.target_left.idx.to_string()),
                        cfg.target_left.idx,
                    );
                    let target_right = DataCell::new_without_data(
                        cfg.target_right.target_type,
                        cfg.target_right
                            .header
                            .unwrap_or_else(|| cfg.target_right.idx.to_string()),
                        cfg.target_right.idx,
                    );
                    match cfg.spec {
                        SplitterType::SeparatorChar { char } => {
                            transrichers.push(Box::new(SplitItemAtIdx {
                                delete_source_item: cfg.delete_after_split,
                                idx: cfg.idx,
                                splitter: SplitDataCellUsingValueSplit {
                                    splitter: ValueStringSeparatorCharSplit {
                                        sep_char: char,
                                        split_none: true, // TODO: config
                                    },
                                    target_left: target_left,
                                    target_right: target_right,
                                },
                            }));
                        }
                        SplitterType::Pattern { pattern } => {
                            transrichers.push(Box::new(SplitItemAtIdx {
                                delete_source_item: cfg.delete_after_split,
                                idx: cfg.idx,
                                splitter: SplitDataCellUsingValueSplit {
                                    splitter: ValueStringRegexPairSplit::from(pattern, true)?,
                                    target_left: target_left,
                                    target_right: target_right,
                                },
                            }));
                        }
                    }
                }
                TransformerConfig::AddItem { cfg } => {
                    match cfg.spec {
                        AddItemType::Meta { key } => todo!(),
                        AddItemType::Static { value } => {
                            // CAUTION: this only supports the standard conversion! (Meaning, non-standard date/time formats are not supported here)
                            transrichers.push(Box::new(AddItemStatic {
                                0: DataCell::new(
                                    cfg.target.target_type.clone(),
                                    cfg.target
                                        .header
                                        .unwrap_or_else(|| cfg.target.idx.to_string()),
                                    cfg.target.idx,
                                    Value::from_string_with_templ(&value, &cfg.target.target_type)?,
                                ),
                            }));
                        }
                        AddItemType::Runtime { rt_value } => todo!(),
                    }
                }
            };
        }

        // TODO: ordering

        Ok(TransrichPass {
            transformer: transrichers,
            order: None,
        })
    }
}
