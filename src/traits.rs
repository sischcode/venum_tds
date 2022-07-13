use venum::venum::ValueType;

use crate::errors::Result;

pub trait VDataContainerItem {
    type DATA;

    fn get_type_info(&self) -> &ValueType;
    fn set_type_info(&mut self, type_info: ValueType);

    fn get_idx(&self) -> usize;
    fn set_idx(&mut self, idx: usize);

    fn get_name(&self) -> &str;
    fn set_name(&mut self, name: &str);

    fn get_data(&self) -> &Self::DATA;
    fn get_data_mut(&mut self) -> &mut Self::DATA;
    fn set_data(&mut self, data: Self::DATA);
}

pub trait VDataContainer {
    type ITEM: VDataContainerItem;

    fn get_by_idx(&self, idx: usize) -> Option<&Self::ITEM>;
    fn get_by_idx_mut(&mut self, idx: usize) -> Option<&mut Self::ITEM>;

    fn del_by_idx(&mut self, idx: usize) -> Result<Self::ITEM>;
    fn add(&mut self, elem: Self::ITEM);

    fn get_by_name(&self, name: &str) -> Option<&Self::ITEM>;
    fn get_by_name_mut(&mut self, name: &str) -> Option<&mut Self::ITEM>;
}
