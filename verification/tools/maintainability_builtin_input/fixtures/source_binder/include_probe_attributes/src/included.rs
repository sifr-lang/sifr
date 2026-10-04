#[derive(Debug)]
pub struct IncludedBinder<'a> { pub value: &'a u8 }
impl<'a> IncludedBinder<'a> { pub fn get(&self) -> &'a u8 { self.value } }
