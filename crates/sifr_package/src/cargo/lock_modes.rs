#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CargoLockMode {
    #[default]
    Normal,
    Locked,
    Offline,
    Frozen,
}

impl CargoLockMode {
    #[must_use]
    pub const fn is_network_disallowed(self) -> bool {
        matches!(self, Self::Offline | Self::Frozen)
    }

    #[must_use]
    pub const fn is_lock_mutation_disallowed(self) -> bool {
        matches!(self, Self::Locked | Self::Frozen)
    }
}
