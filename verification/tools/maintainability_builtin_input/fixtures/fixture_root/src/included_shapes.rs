#[derive(Debug, Clone, PartialEq)]
pub struct Included<'a> { pub value: &'a str }

pub struct LocalContainer;
impl LocalContainer {
    pub fn construct() {
        #[derive(Default)]
        struct LocalShape { flag: bool }
        let _ = LocalShape::default();
    }
}
