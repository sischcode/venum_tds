use crate::errors::Result;

pub trait Indexed {
    fn get_idx(&self) -> usize;
    fn set_idx(&mut self, idx: usize);
    fn get_by_idx(&self, idx: usize) -> Result<&Self>;
    fn get_by_idx_mut(&mut self, idx: usize) -> Result<&mut Self>;
}

pub trait Named {
    fn get_name(&self) -> &str;
    fn set_name(&mut self, name: &str);
    fn get_by_name(&self, name: &str) -> Result<&Self>;
    fn get_by_name_mut(&mut self, name: &str) -> Result<&mut Self>;
}
