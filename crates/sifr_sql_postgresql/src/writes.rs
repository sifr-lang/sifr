use crate::analysis::{
    AnalysisContext, AnalyzedStatement, PostgresAnalysisError, ResultFact, ScopeBinding,
    ScopeFrame, write_error,
};
use crate::ast::{
    Assignment, ConflictAction, DeleteStatement, ExpressionKind, InsertStatement, UpdateStatement,
};
use crate::catalog::{CatalogColumn, CatalogRelation};
use crate::scope::binding_for_relation;
use sifr_sql_contract::{Cardinality, ObjectId, QueryEffect};
use std::collections::BTreeSet;

impl AnalysisContext<'_> {
    pub(crate) fn analyze_insert(
        &mut self,
        insert: &InsertStatement,
    ) -> Result<AnalyzedStatement, PostgresAnalysisError> {
        let relation = self
            .catalog
            .relation(&insert.relation)
            .map_err(|diagnostic| PostgresAnalysisError { diagnostic })?
            .clone();
        let target_columns = if insert.columns.is_empty() {
            relation.column_order.clone()
        } else {
            insert.columns.clone()
        };
        for required in relation
            .columns
            .values()
            .filter(|column| !column.nullable && !column.has_default && !column.generated)
        {
            if !target_columns.contains(&required.name) {
                return Err(write_error(format!(
                    "INSERT omits required column '{}.{}'",
                    relation.identity, required.name
                )));
            }
        }
        for row in &insert.rows {
            if row.len() != target_columns.len() {
                return Err(write_error(
                    "INSERT row width does not match its column list",
                ));
            }
            for (expression, column_name) in row.iter().zip(&target_columns) {
                let column = writable_column(&relation, column_name)?;
                let fact = self.infer(expression, &[], Some(&column.database_type))?;
                if fact.nullable && !column.nullable {
                    return Err(write_error(format!(
                        "nullable expression cannot be assigned to '{}.{}'",
                        relation.identity, column.name
                    )));
                }
            }
        }
        if let Some(source) = &insert.source {
            let expected_types = target_columns
                .iter()
                .map(|column_name| {
                    writable_column(&relation, column_name)
                        .map(|column| column.database_type.clone())
                })
                .collect::<Result<Vec<_>, _>>()?;
            let analyzed =
                self.analyze_select_with_expected(source, Vec::new(), Some(&expected_types))?;
            if analyzed.fields.len() != target_columns.len() {
                return Err(write_error(
                    "INSERT SELECT width does not match its column list",
                ));
            }
            for (field, column_name) in analyzed.fields.iter().zip(&target_columns) {
                let column = writable_column(&relation, column_name)?;
                if !self
                    .catalog
                    .can_cast(&field.database_type, &column.database_type, true)
                    || (field.nullable && !column.nullable)
                {
                    return Err(write_error(
                        "INSERT SELECT value is not assignment-compatible",
                    ));
                }
            }
        }
        let target_binding = binding_for_relation(&relation, None);
        if let Some(conflict) = &insert.conflict {
            let target = conflict
                .target_columns
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>();
            let matches_unique = relation
                .unique_sets
                .iter()
                .any(|columns| columns.iter().cloned().collect::<BTreeSet<_>>() == target);
            if !target.is_empty() && target != relation.primary_key && !matches_unique {
                return Err(write_error("ON CONFLICT target is not a unique key"));
            }
            if let Some(predicate) = &conflict.target_predicate {
                self.require_boolean(
                    predicate,
                    std::slice::from_ref(&ScopeFrame {
                        bindings: vec![target_binding.clone()],
                    }),
                )?;
            }
            if conflict.action == ConflictAction::Update {
                let conflict_frames = vec![ScopeFrame {
                    bindings: vec![
                        target_binding.clone(),
                        ScopeBinding {
                            alias: "excluded".to_string(),
                            relation: None,
                            columns: relation.columns.clone(),
                        },
                    ],
                }];
                self.check_assignments(&relation, &conflict.assignments, &conflict_frames)?;
                if let Some(predicate) = &conflict.update_predicate {
                    self.require_boolean(predicate, &conflict_frames)?;
                }
            }
        }
        let frames = vec![ScopeFrame {
            bindings: vec![target_binding],
        }];
        let fields = self.result_fields(&insert.returning, &frames)?;
        Ok(write_analysis(&relation, fields, self.referenced.clone()))
    }

    pub(crate) fn analyze_update(
        &mut self,
        update: &UpdateStatement,
    ) -> Result<AnalyzedStatement, PostgresAnalysisError> {
        let relation = self
            .catalog
            .relation(&update.relation)
            .map_err(|diagnostic| PostgresAnalysisError { diagnostic })?
            .clone();
        self.referenced.insert(relation.identity.clone());
        let mut frame = ScopeFrame {
            bindings: vec![binding_for_relation(&relation, update.alias.as_deref())],
        };
        for item in &update.from {
            self.add_from_item(item, &[], &mut frame)?;
        }
        let frames = vec![frame];
        self.check_assignments(&relation, &update.assignments, &frames)?;
        if let Some(predicate) = &update.predicate {
            self.require_boolean(predicate, &frames)?;
        }
        let fields = self.result_fields(&update.returning, &frames)?;
        Ok(write_analysis(&relation, fields, self.referenced.clone()))
    }

    pub(crate) fn analyze_delete(
        &mut self,
        delete: &DeleteStatement,
    ) -> Result<AnalyzedStatement, PostgresAnalysisError> {
        let relation = self
            .catalog
            .relation(&delete.relation)
            .map_err(|diagnostic| PostgresAnalysisError { diagnostic })?
            .clone();
        self.referenced.insert(relation.identity.clone());
        let mut frame = ScopeFrame {
            bindings: vec![binding_for_relation(&relation, delete.alias.as_deref())],
        };
        for item in &delete.using_relations {
            self.add_from_item(item, &[], &mut frame)?;
        }
        let frames = vec![frame];
        if let Some(predicate) = &delete.predicate {
            self.require_boolean(predicate, &frames)?;
        }
        let fields = self.result_fields(&delete.returning, &frames)?;
        Ok(write_analysis(&relation, fields, self.referenced.clone()))
    }

    fn check_assignments(
        &mut self,
        relation: &CatalogRelation,
        assignments: &[Assignment],
        frames: &[ScopeFrame],
    ) -> Result<(), PostgresAnalysisError> {
        let mut assigned = BTreeSet::new();
        for assignment in assignments {
            if !assigned.insert(&assignment.column) {
                return Err(write_error(format!(
                    "column '{}' is assigned more than once",
                    assignment.column
                )));
            }
            let column = writable_column(relation, &assignment.column)?;
            if !matches!(assignment.value.kind, ExpressionKind::Default) {
                let fact = self.infer(&assignment.value, frames, Some(&column.database_type))?;
                if fact.nullable && !column.nullable {
                    return Err(write_error(format!(
                        "nullable expression cannot be assigned to '{}.{}'",
                        relation.identity, column.name
                    )));
                }
            }
        }
        Ok(())
    }
}

fn writable_column<'a>(
    relation: &'a CatalogRelation,
    name: &str,
) -> Result<&'a CatalogColumn, PostgresAnalysisError> {
    let column = relation.columns.get(name).ok_or_else(|| {
        write_error(format!(
            "unknown writable column '{}.{name}'",
            relation.identity
        ))
    })?;
    if column.generated {
        return Err(write_error(format!(
            "generated column '{}.{name}' cannot be assigned",
            relation.identity
        )));
    }
    Ok(column)
}

fn write_analysis(
    relation: &CatalogRelation,
    fields: Vec<ResultFact>,
    mut referenced: BTreeSet<ObjectId>,
) -> AnalyzedStatement {
    referenced.insert(relation.identity.clone());
    AnalyzedStatement {
        cardinality: if fields.is_empty() {
            Cardinality::ZERO
        } else {
            Cardinality::MANY
        },
        fields,
        effect: QueryEffect::Write,
        referenced,
        affected: BTreeSet::from([relation.identity.clone()]),
        flags: BTreeSet::new(),
    }
}
