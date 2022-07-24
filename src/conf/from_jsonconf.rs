use std::collections::HashMap;

use super::jsonconf::ConfigRoot;
use venum::venum::Value;

use crate::{
    conf::jsonconf::{AddItemType, SplitterType, TransformEnrichPassConfig, TransformerConfig},
    data_cell::DataCell,
    errors::{Result, VenumTdsError},
    transform::{
        data_cell::splitting::SplitDataCellUsingValueSplit,
        data_cell_row::{
            mutate::{
                AddItemRuntime, AddItemRuntimeSingleton, AddItemStatic, DeleteItemAtIdx,
                MutateItemIdx, SplitItemAtIdx, TransrichDataCellRowInplace,
            },
            transrich_pass::{TransrichPass, TransrichPassesConfig},
        },
        value::spliting::{ValueStringRegexPairSplit, ValueStringSeparatorCharSplit},
    },
};

impl TryFrom<(TransformEnrichPassConfig, &Option<HashMap<String, Value>>)> for TransrichPass {
    type Error = VenumTdsError;

    fn try_from(
        tuple: (TransformEnrichPassConfig, &Option<HashMap<String, Value>>),
    ) -> Result<Self> {
        let (tepc, enrich_map) = tuple;

        let mut transrichers: Vec<Box<dyn TransrichDataCellRowInplace>> =
            Vec::with_capacity(tepc.transformers.len());

        for tc in tepc.transformers {
            match tc {
                TransformerConfig::DeleteItems { cfg } => {
                    for i in cfg {
                        transrichers.push(Box::new(DeleteItemAtIdx(i)));
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
                                    target_left,
                                    target_right,
                                },
                            }));
                        }
                        SplitterType::Pattern { pattern } => {
                            transrichers.push(Box::new(SplitItemAtIdx {
                                delete_source_item: cfg.delete_after_split,
                                idx: cfg.idx,
                                splitter: SplitDataCellUsingValueSplit {
                                    splitter: ValueStringRegexPairSplit::new(pattern, true)?,
                                    target_left,
                                    target_right,
                                },
                            }));
                        }
                    }
                }
                TransformerConfig::AddItem { cfg } => {
                    match cfg.spec {
                        AddItemType::Meta { key } => {
                            if enrich_map.is_none() {
                                return Err(VenumTdsError::Generic { msg: String::from("No metadata / enrichment map available, but at least needed for one transrichment") });
                            }
                            transrichers.push(Box::new(AddItemStatic(DataCell::new(
                                cfg.target.target_type.clone(),
                                cfg.target
                                    .header
                                    .unwrap_or_else(|| cfg.target.idx.to_string()),
                                cfg.target.idx,
                                enrich_map
                                    .as_ref()
                                    .unwrap()
                                    .get(&key)
                                    .ok_or_else(|| VenumTdsError::Generic {
                                        msg: format!(
                                            "No value for key={} in metadata / enrichment map",
                                            &key
                                        ),
                                    })?
                                    .to_owned(),
                            ))));
                        }
                        AddItemType::Static { value } => {
                            // CAUTION: this only supports the standard conversion! (Meaning, non-standard date/time formats are not supported here)
                            transrichers.push(Box::new(AddItemStatic(DataCell::new(
                                cfg.target.target_type.clone(),
                                cfg.target
                                    .header
                                    .unwrap_or_else(|| cfg.target.idx.to_string()),
                                cfg.target.idx,
                                Value::from_str_and_type(&value, &cfg.target.target_type)?,
                            ))));
                        }
                        AddItemType::Runtime {
                            rt_value,
                            as_singleton,
                        } => {
                            if as_singleton.unwrap_or(false) {
                                transrichers.push(Box::new(AddItemRuntimeSingleton::new(
                                    cfg.target.header,
                                    cfg.target.idx,
                                    rt_value,
                                )));
                            } else {
                                transrichers.push(Box::new(AddItemRuntime {
                                    header: cfg.target.header,
                                    idx: cfg.target.idx,
                                    rtv: rt_value,
                                }));
                            }
                        }
                    }
                }
            };
        }

        let mut ordering_opt: Option<Vec<Box<dyn TransrichDataCellRowInplace>>> = None;
        if let Some(order_items) = tepc.order_items {
            let mut ordering: Vec<Box<dyn TransrichDataCellRowInplace>> =
                Vec::with_capacity(order_items.len());

            for o in order_items {
                ordering.push(Box::new(MutateItemIdx {
                    from: o.from,
                    to: o.to,
                }));
            }
            ordering_opt = Some(ordering);
        }

        Ok(TransrichPass {
            transformer: transrichers,
            order: ordering_opt,
        })
    }
}

impl TryFrom<TransformEnrichPassConfig> for TransrichPass {
    type Error = VenumTdsError;
    fn try_from(tepc: TransformEnrichPassConfig) -> Result<Self> {
        TransrichPass::try_from((tepc, &None))
    }
}

impl TryFrom<(ConfigRoot, &Option<HashMap<String, Value>>)> for TransrichPassesConfig {
    type Error = VenumTdsError;

    fn try_from(tuple: (ConfigRoot, &Option<HashMap<String, Value>>)) -> Result<Self> {
        let (mut config, enrich_map) = tuple;
        if config.0.is_empty() {
            return Ok(TransrichPassesConfig { passes: Vec::new() });
        }

        let mut v: Vec<TransrichPass> = Vec::with_capacity(config.0.len());
        while !config.0.is_empty() {
            let tepc = config.0.pop().expect("never empty!");
            let trp = TransrichPass::try_from((tepc, enrich_map))?;
            v.push(trp)
        }
        v.reverse();
        Ok(TransrichPassesConfig { passes: v })
    }
}

impl TryFrom<ConfigRoot> for TransrichPassesConfig {
    type Error = VenumTdsError;
    fn try_from(config: ConfigRoot) -> Result<Self> {
        TransrichPassesConfig::try_from((config, &None))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use venum::venum::{Value, ValueType};

    use crate::{
        conf::jsonconf::{
            AddItemConfig, AddItemType, ItemTargetConfig, OrderItemsEntry, SplitItemConfig,
            SplitterType, TransformEnrichPassConfig, TransformerConfig,
        },
        data_cell::DataCell,
        transform::{
            data_cell::splitting::*,
            data_cell_row::{mutate::*, transrich_pass::TransrichPass},
            value::spliting::*,
        },
    };

    #[test]
    fn test_try_from_tepc_one_pass() {
        let exp = TransrichPass {
            transformer: vec![
                Box::new(DeleteItemAtIdx(0)),
                Box::new(DeleteItemAtIdx(1)),
                Box::new(SplitItemAtIdx {
                    delete_source_item: true,
                    idx: 2,
                    splitter: SplitDataCellUsingValueSplit {
                        splitter: ValueStringRegexPairSplit::new(
                            String::from("(\\d+\\.\\d+) \\(([[:alpha:]].+)\\)"),
                            true,
                        )
                        .unwrap(),
                        target_left: DataCell::new_without_data(
                            ValueType::Float32,
                            String::from("from_2_left"),
                            10,
                        ),
                        target_right: DataCell::new_without_data(
                            ValueType::String,
                            String::from("from_2_right"),
                            11,
                        ),
                    },
                }),
                Box::new(SplitItemAtIdx {
                    delete_source_item: true,
                    idx: 3,
                    splitter: SplitDataCellUsingValueSplit {
                        splitter: ValueStringSeparatorCharSplit {
                            sep_char: ';',
                            split_none: true,
                        },
                        target_left: DataCell::new_without_data(
                            ValueType::Float32,
                            String::from("from_3_left"),
                            20,
                        ),
                        target_right: DataCell::new_without_data(
                            ValueType::String,
                            String::from("from_3_right"),
                            21,
                        ),
                    },
                }),
                Box::new(AddItemStatic(DataCell::new(
                    ValueType::String,
                    String::from("Region"),
                    22,
                    Value::String(String::from("Europe")),
                ))),
                Box::new(AddItemStatic(DataCell::new(
                    ValueType::Float32,
                    String::from("Magic Number"),
                    23,
                    Value::Float32(1.123),
                ))),
                // // We can't rely test this, because of the dynamic nature...
                // Box::new(AddItemRuntime {
                //     header: Some(String::from("Runtime DateTime 1")),
                //     idx: 24,
                //     rtv: RuntimeValue::CurrentDateTimeUtcAsFixedOffset,
                // }),
                // Box::new(AddItemRuntimeSingleton::new(
                //     Some(String::from("Runtime DateTime 2")),
                //     24,
                //     RuntimeValue::CurrentDateTimeUtcAsFixedOffset,
                // )),
                Box::new(AddItemStatic(DataCell::new(
                    ValueType::Int32,
                    String::from("Account Id"),
                    26,
                    Value::Int32(1000),
                ))),
            ],
            order: Some(vec![
                Box::new(MutateItemIdx { from: 10, to: 0 }),
                Box::new(MutateItemIdx { from: 11, to: 1 }),
                Box::new(MutateItemIdx { from: 20, to: 2 }),
                Box::new(MutateItemIdx { from: 21, to: 3 }),
                Box::new(MutateItemIdx { from: 22, to: 4 }),
                Box::new(MutateItemIdx { from: 23, to: 5 }),
                Box::new(MutateItemIdx { from: 24, to: 6 }),
                Box::new(MutateItemIdx { from: 25, to: 7 }),
                Box::new(MutateItemIdx { from: 26, to: 8 }),
            ]),
        };

        let dsl_fmt = TransformEnrichPassConfig {
            comment: Some(String::from("pass1")),
            transformers: vec![
                TransformerConfig::DeleteItems { cfg: vec![0, 1] },
                TransformerConfig::SplitItem {
                    cfg: SplitItemConfig {
                        idx: 2,
                        spec: SplitterType::Pattern {
                            pattern: String::from("(\\d+\\.\\d+) \\(([[:alpha:]].+)\\)"),
                        },
                        delete_after_split: true,
                        target_left: ItemTargetConfig {
                            idx: 10,
                            header: Some(String::from("from_2_left")),
                            target_type: ValueType::Float32,
                        },
                        target_right: ItemTargetConfig {
                            idx: 11,
                            header: Some(String::from("from_2_right")),
                            target_type: ValueType::String,
                        },
                    },
                },
                TransformerConfig::SplitItem {
                    cfg: SplitItemConfig {
                        idx: 3,
                        spec: SplitterType::SeparatorChar { char: ';' },
                        delete_after_split: true,
                        target_left: ItemTargetConfig {
                            idx: 20,
                            header: Some(String::from("from_3_left")),
                            target_type: ValueType::Float32,
                        },
                        target_right: ItemTargetConfig {
                            idx: 21,
                            header: Some(String::from("from_3_right")),
                            target_type: ValueType::String,
                        },
                    },
                },
                TransformerConfig::AddItem {
                    cfg: AddItemConfig {
                        spec: AddItemType::Static {
                            value: String::from("Europe"),
                        },
                        target: ItemTargetConfig {
                            idx: 22,
                            header: Some(String::from("Region")),
                            target_type: ValueType::String,
                        },
                    },
                },
                TransformerConfig::AddItem {
                    cfg: AddItemConfig {
                        spec: AddItemType::Static {
                            value: String::from("1.123"),
                        },
                        target: ItemTargetConfig {
                            idx: 23,
                            header: Some(String::from("Magic Number")),
                            target_type: ValueType::Float32,
                        },
                    },
                },
                // // We can't rely test this, because of the dynamic nature...
                // TransformerConfig::AddItem {
                //     cfg: AddItemConfig {
                //         spec: AddItemType::Runtime {
                //             rt_value: RuntimeValue::CurrentDateTimeUtcAsFixedOffset,
                //             as_singleton: Some(false),
                //         },
                //         target: ItemTargetConfig {
                //             idx: 24,
                //             header: Some(String::from("Runtime DateTime 1")),
                //             target_type: ValueType::DateTime,
                //         },
                //     },
                // },
                // TransformerConfig::AddItem {
                //     cfg: AddItemConfig {
                //         spec: AddItemType::Runtime {
                //             rt_value: RuntimeValue::CurrentDateTimeUtcAsFixedOffset,
                //             as_singleton: Some(true),
                //         },
                //         target: ItemTargetConfig {
                //             idx: 25,
                //             header: Some(String::from("Runtime DateTime 2")),
                //             target_type: ValueType::DateTime, // strictly speaking, I thing we don't need the type info here
                //         },
                //     },
                // },
                TransformerConfig::AddItem {
                    cfg: AddItemConfig {
                        spec: AddItemType::Meta {
                            key: String::from("account_id"),
                        },
                        target: ItemTargetConfig {
                            idx: 26,
                            header: Some(String::from("Account Id")),
                            target_type: ValueType::Int32, // TODO: strictly speaking, I thing we don't need the type info here
                        },
                    },
                },
            ],
            order_items: Some(vec![
                OrderItemsEntry { from: 10, to: 0 },
                OrderItemsEntry { from: 11, to: 1 },
                OrderItemsEntry { from: 20, to: 2 },
                OrderItemsEntry { from: 21, to: 3 },
                OrderItemsEntry { from: 22, to: 4 },
                OrderItemsEntry { from: 23, to: 5 },
                OrderItemsEntry { from: 24, to: 6 },
                OrderItemsEntry { from: 25, to: 7 },
                OrderItemsEntry { from: 26, to: 8 },
            ]),
        };

        let mut metadata: HashMap<String, Value> = HashMap::with_capacity(1);
        metadata.insert(String::from("account_id"), Value::Int32(1000));

        let test_pass = TransrichPass::try_from((dsl_fmt, &Some(metadata))).unwrap();

        assert_eq!(format!("{:?}", exp), format!("{:?}", test_pass));
    }
}
