#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CargoAction {
    Metadata,
    Fetch,
    Package,
    Publish,
    Vendor,
    Add,
    Remove,
    Update,
}

impl CargoAction {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Metadata => "metadata",
            Self::Fetch => "fetch",
            Self::Package => "package",
            Self::Publish => "publish",
            Self::Vendor => "vendor",
            Self::Add => "add",
            Self::Remove => "remove",
            Self::Update => "update",
        }
    }
}
