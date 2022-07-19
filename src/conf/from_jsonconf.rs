use crate::transform::data_cell_row::mutate::SplitItemAtIdx;

use super::jsonconf::{
    SplitItemConfig, SplitterType, TransformEnrichPassConfig, TransformerConfig,
};

impl TryFrom<SplitItemConfig> for Box<SplitItemAtIdx<ValueStringSeparatorCharSplit>> {
    type Error = VenumTdsError;

    fn try_from(cfg: &SplitItemConfig) -> Result<Self> {
        let spl = match cfg.spec {
            SplitterType::SeparatorChar { char } => {
                Ok(ValueStringSeparatorCharSplit {
                    sep_char: char,
                    split_none: true, // TODO: config
                })
            }
            SplitterType::Pattern { pattern: _ } => Err(VenumTdsTransRichError::Generic {
                msg: String::from("shit"),
            }),
        }?;
        Ok(Box::new(SplitItemAtIdx {
            delete_source_item: cfg.delete_after_split,
            idx: cfg.idx,
            splitter: spl,
            target_left: (
                Value::from(cfg.target_left.target_type),
                cfg.target_left.idx,
                String::from("TODO"), // TODO
            ),
            target_right: (
                Value::from(cfg.target_right.target_type),
                cfg.target_right.idx,
                String::from("TODO"), // TODO
            ),
        }))
    }
}

impl<C> TryFrom<TransformEnrichPassConfig> for TransrichPass<C>
where
    C: VDataContainer,
    <C as VDataContainer>::ITEM: Default + SplitUsing2<ValueStringSeparatorCharSplit>,
{
    type Error = VenumTdsTransRichError;

    fn try_from(tepc: TransformEnrichPassConfig) -> Result<Self> {
        let mut vt: Vec<Box<dyn TransrichContainerInplace<C>>> = Vec::new();

        for tc in tepc.transformers {
            match tc {
                TransformerConfig::DeleteItems { cfg } => todo!(),
                TransformerConfig::SplitItem { cfg } => match cfg.spec {
                    SplitterType::SeparatorChar { char } => {
                        let sp: Box<SplitItemAtIdx<ValueStringSeparatorCharSplit>> =
                            cfg.try_into()?;
                        vt.push(sp);
                    }
                    SplitterType::Pattern { pattern } => todo!(),
                },
                TransformerConfig::AddItem { cfg } => todo!(),
            };
        }

        // tepc.transformers.iter().map(|tec| match tec {
        //     TransformerConfig::DeleteItems { cfg } => todo!(),
        //     TransformerConfig::SplitItem { cfg } => match cfg.spec {
        //         SplitterType::SeparatorChar { char } => {
        //             let sp: Box<SplitItemAtIdx<ValueStringSeparatorCharSplit>> = cfg.try_into()?;
        //             vt.push(sp);
        //         }
        //         SplitterType::Pattern { pattern } => todo!(),
        //     },
        //     TransformerConfig::AddItem { cfg } => todo!(),
        // });

        Ok(())
    }
}
