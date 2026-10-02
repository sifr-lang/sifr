// Real independent declarations, including a Unicode source-offset prefix: λ.
#![allow(dead_code)]
pub fn first<F>(f: F) -> u8
where
    F: for<'x> FnOnce(&'x u8) -> u8,
{
    f(&1)
}
pub fn second<G>(g: G) -> u8
where
    G: for<'x> FnOnce(&'x u8) -> u8,
{
    g(&2)
}
