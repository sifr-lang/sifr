pub trait Contract { fn mark(&self); }
#[derive(Debug)]
pub struct Container<'parent> { pub value: &'parent u8 }
impl<'parent> Contract for Container<'parent> { fn mark(&self) {} }
impl<'parent> Container<'parent> {
    pub fn apply<'method, F>(&'parent self, f: F, value: &'method u8) -> u8
    where 'parent: 'method, F: for<'bound> FnOnce(&'bound u8) -> u8 { f(value) }
}
pub fn independent<F>(f: F) -> u8 where F: for<'bound> FnOnce(&'bound u8) -> u8 { f(&1) }
