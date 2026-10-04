// Unicode source: café λ
pub trait Formeλ { fn mark(&self); }
#[derive(Debug)]
pub struct Café<'α> { pub value: &'α u8 }
impl<'α> Formeλ for Café<'α> { fn mark(&self) {} }
impl<'α> Café<'α> {
    pub fn apply<'β, F>(&'α self, f: F, value: &'β u8) -> u8
    where 'α: 'β, F: for<'ξ> FnOnce(&'ξ u8) -> u8 { f(value) }
}
pub fn independent<F>(f: F) -> u8 where F: for<'ξ> FnOnce(&'ξ u8) -> u8 { f(&1) }
