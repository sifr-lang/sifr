use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CargoCommandPlan {
    pub program: String,
    pub args: Vec<String>,
    pub current_dir: PathBuf,
}

impl CargoCommandPlan {
    #[must_use]
    pub fn metadata(current_dir: PathBuf) -> Self {
        Self {
            program: "cargo".to_string(),
            args: vec![
                "metadata".to_string(),
                "--format-version".to_string(),
                "1".to_string(),
            ],
            current_dir,
        }
    }
}
