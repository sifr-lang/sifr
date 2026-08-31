use super::{
    MigrationCompileError, MigrationCompileErrorKind, MigrationGraphDefinition, MigrationNodeId,
};
use std::collections::{BTreeMap, BTreeSet};

pub fn topological_order(
    graph: &MigrationGraphDefinition,
) -> Result<Vec<MigrationNodeId>, MigrationCompileError> {
    if graph.baselines.is_empty() || graph.migrations.is_empty() {
        return Err(error("migration graphs require a baseline and a migration"));
    }
    for (id, baseline) in &graph.baselines {
        if id != &baseline.id || graph.migrations.contains_key(id) {
            return Err(error(
                "migration node identities must be unique and canonical",
            ));
        }
    }
    let mut children = BTreeMap::<MigrationNodeId, BTreeSet<MigrationNodeId>>::new();
    let mut indegree = BTreeMap::<MigrationNodeId, usize>::new();
    for (id, migration) in &graph.migrations {
        if id != &migration.id || migration.parents.is_empty() {
            return Err(error(
                "each migration requires its canonical identity and a parent",
            ));
        }
        if migration.input_fingerprints.keys().collect::<BTreeSet<_>>()
            != migration.parents.iter().collect::<BTreeSet<_>>()
        {
            return Err(error(
                "migration input fingerprints must match its parent set exactly",
            ));
        }
        indegree.insert(id.clone(), 0);
    }
    for (id, migration) in &graph.migrations {
        for parent in &migration.parents {
            if !graph.baselines.contains_key(parent) && !graph.migrations.contains_key(parent) {
                return Err(error(format!(
                    "migration '{id}' references unknown parent '{parent}'"
                )));
            }
            children
                .entry(parent.clone())
                .or_default()
                .insert(id.clone());
            if graph.migrations.contains_key(parent) {
                let count = indegree
                    .get_mut(id)
                    .ok_or_else(|| error("missing migration node"))?;
                *count += 1;
            }
        }
    }
    let mut ready = indegree
        .iter()
        .filter(|(_, count)| **count == 0)
        .map(|(id, _)| id.clone())
        .collect::<BTreeSet<_>>();
    let mut order = Vec::with_capacity(graph.migrations.len());
    while let Some(id) = ready.pop_first() {
        order.push(id.clone());
        for child in children.get(&id).into_iter().flatten() {
            let count = indegree
                .get_mut(child)
                .ok_or_else(|| error("migration child is not canonical"))?;
            *count = count
                .checked_sub(1)
                .ok_or_else(|| error("migration graph indegree underflow"))?;
            if *count == 0 {
                ready.insert(child.clone());
            }
        }
    }
    if order.len() != graph.migrations.len() {
        return Err(error("migration graph contains a cycle"));
    }
    let heads = graph
        .migrations
        .keys()
        .filter(|id| children.get(*id).is_none_or(BTreeSet::is_empty))
        .collect::<Vec<_>>();
    if heads.len() != 1 {
        return Err(error("migration graph must have one explicit head"));
    }
    for baseline in graph.baselines.keys() {
        if !reaches_head(baseline, heads[0], &children) {
            return Err(error(format!(
                "baseline '{baseline}' does not reach the migration head"
            )));
        }
    }
    Ok(order)
}

fn reaches_head(
    start: &MigrationNodeId,
    head: &MigrationNodeId,
    children: &BTreeMap<MigrationNodeId, BTreeSet<MigrationNodeId>>,
) -> bool {
    let mut pending = vec![start];
    let mut seen = BTreeSet::new();
    while let Some(node) = pending.pop() {
        if node == head {
            return true;
        }
        if !seen.insert(node.clone()) {
            continue;
        }
        pending.extend(children.get(node).into_iter().flatten());
    }
    false
}

fn error(message: impl Into<String>) -> MigrationCompileError {
    MigrationCompileError::new(MigrationCompileErrorKind::InvalidGraph, message)
}
