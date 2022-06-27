use venum::venum::Value;

use crate::errors::Result;

pub trait DataEntry {
    type D;

    fn get_type_info(&self) -> &Value;

    fn get_idx(&self) -> usize;
    fn set_idx(&mut self, idx: usize);

    fn get_name(&self) -> &str;
    fn set_name(&mut self, name: &str);

    fn get_data(&self) -> &Self::D;
    fn set_data(&mut self, data: Self::D);
}

pub trait DataContainer {
    type T: DataEntry;

    fn get_by_idx(&self, idx: usize) -> Option<&Self::T>;
    fn get_by_idx_mut(&mut self, idx: usize) -> Option<&mut Self::T>;

    fn del_by_idx(&mut self, idx: usize) -> Result<Self::T>;
    fn add(&mut self, elem: Self::T);

    fn get_by_name(&self, name: &str) -> Option<&Self::T>;
    fn get_by_name_mut(&mut self, name: &str) -> Option<&mut Self::T>;
}
