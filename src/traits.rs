use venum::venum::Value;

pub trait TypeInfo {
    fn get_type_info(&self) -> &Value;
}

pub trait Indexed {
    fn get_idx(&self) -> usize;
    fn set_idx(&mut self, idx: usize);
}

pub trait IndexAccess<T: Indexed> {
    fn get_by_idx(&self, idx: usize) -> Option<&T>;
    fn get_by_idx_mut(&mut self, idx: usize) -> Option<&mut T>;
}

pub trait Named {
    fn get_name(&self) -> &str;
    fn set_name(&mut self, name: &str);
}

pub trait NameAccess<T: Named> {
    fn get_by_name(&self, name: &str) -> Option<&T>;
    fn get_by_name_mut(&mut self, name: &str) -> Option<&mut T>;
}
