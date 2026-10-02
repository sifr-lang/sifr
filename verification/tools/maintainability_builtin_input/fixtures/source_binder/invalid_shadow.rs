#![allow(dead_code)]
pub fn invalid<'x, F>(f: F)
where
    F: for<'x> FnOnce(&'x u8),
{
    f(&1);
}
