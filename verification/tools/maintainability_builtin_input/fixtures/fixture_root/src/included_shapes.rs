// Original ordinary comments and their bytes must remain in source evidence.
#[derive(Debug, Clone, PartialEq)]
pub struct Included<'a> { /* 'a :: tokens inside this comment are trivia */ pub value: &'a str }

pub struct LocalContainer;
impl LocalContainer {
    pub fn construct() {
        #[derive(Default)]
        struct LocalShape { flag: bool }
        let _ = LocalShape::default();
    }
}
