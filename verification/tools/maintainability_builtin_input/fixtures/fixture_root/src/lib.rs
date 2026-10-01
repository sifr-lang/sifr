#[derive(Debug, Clone, PartialEq)]
pub struct Record { pub text: String, pub scalar: i32 }
#[derive(Debug, Clone, PartialEq)]
pub struct Tuple(pub String, pub i32);
#[derive(Debug, Clone, PartialEq)]
pub struct Unit;
#[derive(Debug, Clone, PartialEq)]
pub enum Variants { Unit, Tuple(String, i32), Record { text: String, number: i32 } }
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct CopySensitive { pub value: i32 }
mod left { #[derive(Debug, Clone, PartialEq)] pub struct Same(pub String); }
mod right { #[derive(Debug, Clone, PartialEq)] pub struct Same(pub String); }
macro_rules! nest { () => { #[derive(Debug, Clone, PartialEq)] pub struct Nested { pub first: String, pub second: String } }; }
nest!();
#[derive(Debug, Clone, PartialEq)]
pub struct ExternalFields { pub left: field_origin::External, pub right: field_origin::External }
#[allow(non_snake_case)]
macro_rules! Debug { () => { pub struct LocallyGenerated; }; }
Debug!();
#[derive(Debug, Clone, PartialEq)]
pub struct Generic<T> { pub value: T }

// Source-owned and body-nested impls exercise the preselection universe.
impl Record {}
pub fn local_owner() {
    struct Local;
    impl Local {}
}

pub mod extension;
