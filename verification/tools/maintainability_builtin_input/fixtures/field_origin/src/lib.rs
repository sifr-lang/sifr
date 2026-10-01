pub struct External(pub i32);
impl External {
    pub fn clone(&self) -> bool { false }
    pub fn eq(&self, _other: &Self) -> String { String::new() }
}
impl Clone for External { fn clone(&self) -> Self { Self(self.0) } }
impl PartialEq for External { fn eq(&self, other: &Self) -> bool { self.0 == other.0 } }
impl std::fmt::Debug for External {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str("external trait") }
}
