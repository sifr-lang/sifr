#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DirectProbePolicy {
    ExecuteAll,
    DeferTrustedSysroot,
}

impl DirectProbePolicy {
    pub(super) fn should_execute(self, trusted_sysroot: bool) -> bool {
        self == Self::ExecuteAll || !trusted_sysroot
    }
}

#[cfg(test)]
mod tests {
    use super::DirectProbePolicy;

    #[test]
    fn generated_rust_defers_only_integrity_checked_sysroot_probes() {
        let policy = DirectProbePolicy::DeferTrustedSysroot;

        assert!(!policy.should_execute(true));
        assert!(policy.should_execute(false));
        assert!(DirectProbePolicy::ExecuteAll.should_execute(true));
    }
}
