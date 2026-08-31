use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct RuntimeNeeds {
    flags: HashSet<RuntimeNeed>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum RuntimeNeed {
    FileHandles,
}

impl RuntimeNeeds {
    pub(crate) fn require(&mut self, need: RuntimeNeed) {
        self.flags.insert(need);
    }

    pub(crate) fn contains(&self, need: RuntimeNeed) -> bool {
        self.flags.contains(&need)
    }

    pub(crate) fn file_handles(&self) -> bool {
        self.contains(RuntimeNeed::FileHandles)
    }
}
