pub trait Shape { fn mark(&self); }
#[derive(Debug)]
pub struct Holder<'a> { pub value: &'a u8 }
impl<'a> Shape for Holder<'a> { fn mark(&self) {} }
impl<'a> Holder<'a> {
    pub fn apply<'b, F>(&'a self, f: F, value: &'b u8) -> u8
    where 'a: 'b, F: for<'x> FnOnce(&'x u8) -> u8 { f(value) }
}
pub fn independent<F>(f: F) -> u8 where F: for<'x> FnOnce(&'x u8) -> u8 { f(&1) }
