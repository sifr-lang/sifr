use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangedPathSelection {
    pub paths: Vec<PathBuf>,
}
