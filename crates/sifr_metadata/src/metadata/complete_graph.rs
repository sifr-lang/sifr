//! Per-root structural walks using positions in one immutable directory.
//! No graph verdict is reused: every root keeps its own depth and work budgets.
#[cfg(test)]
use super::RecordId;
use super::validation::anchor_kind;
use super::validation_facts::ValidationFacts;
use super::{MetadataStore, NominalView, Record, Result, err};

pub(super) struct CompleteGraph {
    colors: Vec<u32>,
    generation: u32,
    stack: Vec<(usize, bool, usize)>,
}
impl CompleteGraph {
    pub(super) fn new(records: usize) -> Self {
        Self {
            // Four scratch bytes per already-bounded directory record. This is
            // traversal state, not a retained decoded record or persistent cache.
            colors: vec![0; records],
            generation: 0,
            stack: Vec::new(),
        }
    }
    #[cfg(test)]
    pub(super) fn force_generation_wrap(&mut self) {
        self.generation = u32::MAX - 1;
    }
    pub(super) fn validate(
        &mut self,
        store: &MetadataStore,
        root: usize,
        facts: &mut ValidationFacts,
    ) -> Result<()> {
        if anchor_kind(store.directory.0[root].1.kind) {
            return Ok(());
        }
        self.generation = match self.generation.checked_add(2) {
            Some(generation) => generation,
            None => {
                self.colors.fill(0);
                2
            }
        };
        self.stack.clear();
        self.stack.push((root, false, 0));
        let mut work = 0_u64;
        while let Some((index, exit, depth)) = self.stack.pop() {
            if exit {
                self.colors[index] = self.generation + 1;
                continue;
            }
            let color = self.colors[index];
            if color == self.generation {
                return Err(err("cycle in structural type or semantic body records"));
            }
            if color == self.generation + 1 {
                continue;
            }
            if depth > store.limits.graph_depth {
                return Err(err("record graph exceeds depth limit"));
            }
            let entry = &store.directory.0[index].1;
            work = work
                .checked_add(entry.decoded_bound)
                .ok_or_else(|| err("graph work overflow"))?;
            if work > store.limits.retained_bytes {
                return Err(err("demanded graph exceeds decode work budget"));
            }
            let refs = facts.references(store, index, entry)?;
            self.colors[index] = self.generation;
            self.stack.push((index, true, depth));
            for &(child, kind) in refs.iter().rev() {
                let child = child
                    .filter(|&index| store.directory.0[index].1.kind == kind)
                    .ok_or_else(|| err("missing or wrong-kind graph reference"))?;
                if !anchor_kind(kind) && kind != NominalView::KIND {
                    self.stack.push((child, false, depth + 1));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
impl MetadataStore {
    pub(super) fn validate_graph_with_facts(
        &self,
        root: RecordId,
        facts: &mut ValidationFacts,
    ) -> Result<()> {
        let index = self
            .directory
            .index(&root)
            .ok_or_else(|| err("missing graph root"))?;
        CompleteGraph::new(self.directory.0.len()).validate(self, index, facts)
    }
}
