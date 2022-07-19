use crate::{
    data_cell::DataCell,
    data_cell_row::DataCellRow,
    errors::{ContainerOpsErrors, Result, VenumTdsError},
};

use super::data_cell::splitting::SplitDataCell;

pub trait TransrichDataCellRowInplace {
    fn transrich(&self, container: &mut DataCellRow) -> Result<()>;
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
    fn transrich(&self, data_container: &mut DataCellRow) -> Result<()> {
        let container_entry = data_container.get_by_idx_mut(self.from);
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
    fn transrich(&self, data_container: &mut DataCellRow) -> Result<()> {
        data_container.del_by_idx(self.0).map(|_| ())
    }
}

pub struct AddItem(pub DataCell);
impl TransrichDataCellRowInplace for AddItem {
    fn transrich(&self, data_container: &mut DataCellRow) -> Result<()> {
        data_container.push(self.0.clone());
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
    fn transrich(&self, container: &mut DataCellRow) -> Result<()> {
        let entry = container.get_by_idx_mut(self.idx).ok_or_else(|| {
            VenumTdsError::ContainerOps(ContainerOpsErrors::SplitItemError {
                idx: self.idx,
                msg: format!("Container does not have an entry at idx: {}", self.idx),
            })
        })?;

        let (left, right) = self.splitter.split(entry)?;

        container.push(left);
        container.push(right);
        if self.delete_source_item {
            container.del_by_idx(self.idx).unwrap();
        }
        Ok(())
    }
}
