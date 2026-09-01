use crate::analysis::{AnalysisContext, PostgresAnalysisError, ScopeBinding, ScopeFrame};
use crate::ast::{FromItem, JoinKind};
use crate::cardinality_analysis::make_frame_nullable;
use crate::catalog::CatalogColumn;
use crate::diagnostic::PostgresDiagnosticCode;
use crate::scope::binding_for_relation;
use sifr_sql_contract::ObjectId;

impl AnalysisContext<'_> {
    pub(crate) fn scope_from(
        &mut self,
        items: &[FromItem],
        outer: &[ScopeFrame],
    ) -> Result<ScopeFrame, PostgresAnalysisError> {
        let mut frame = ScopeFrame::default();
        for item in items {
            self.add_from_item(item, outer, &mut frame)?;
        }
        Ok(frame)
    }

    pub(crate) fn add_from_item(
        &mut self,
        item: &FromItem,
        outer: &[ScopeFrame],
        frame: &mut ScopeFrame,
    ) -> Result<(), PostgresAnalysisError> {
        match item {
            FromItem::Relation { name, alias, .. } => {
                if name.len() == 1
                    && let Some(binding) = outer
                        .iter()
                        .rev()
                        .flat_map(|scope| scope.bindings.iter())
                        .find(|binding| binding.alias == name[0] && binding.relation.is_none())
                {
                    let mut binding = binding.clone();
                    if let Some(alias) = alias {
                        binding.alias.clone_from(alias);
                    }
                    frame.bindings.push(binding);
                    return Ok(());
                }
                let relation = self
                    .catalog
                    .relation(name)
                    .map_err(|diagnostic| PostgresAnalysisError { diagnostic })?;
                self.referenced.insert(relation.identity.clone());
                frame
                    .bindings
                    .push(binding_for_relation(relation, alias.as_deref()));
            }
            FromItem::Subquery {
                query,
                alias,
                lateral,
                ..
            } => {
                self.required_capabilities
                    .insert("sql.query.subquery".to_string());
                if *lateral {
                    self.required_capabilities
                        .insert("sql.query.lateral".to_string());
                }
                let mut scopes = outer.to_vec();
                if *lateral {
                    scopes.push(frame.clone());
                }
                let analyzed = self.analyze_select(query, scopes)?;
                frame.bindings.push(ScopeBinding {
                    alias: alias.clone(),
                    relation: None,
                    column_order: analyzed
                        .fields
                        .iter()
                        .map(|field| field.name.clone())
                        .collect(),
                    columns: analyzed
                        .fields
                        .into_iter()
                        .enumerate()
                        .map(|(index, field)| {
                            let identity = ObjectId::new(format!("derived.{alias}.{index}"));
                            (
                                field.name.clone(),
                                CatalogColumn {
                                    identity,
                                    name: field.name,
                                    database_type: field.database_type,
                                    nullable: field.nullable,
                                    has_default: false,
                                    generated: false,
                                    source: None,
                                },
                            )
                        })
                        .collect(),
                });
            }
            FromItem::Join {
                join,
                left,
                right,
                condition,
                using_columns,
                ..
            } => {
                self.required_capabilities
                    .insert("sql.query.join".to_string());
                let mut left_frame = ScopeFrame::default();
                self.add_from_item(left, outer, &mut left_frame)?;
                let mut right_outer = outer.to_vec();
                right_outer.push(left_frame.clone());
                let mut right_frame = ScopeFrame::default();
                self.add_from_item(right, &right_outer, &mut right_frame)?;
                if matches!(join, JoinKind::Right | JoinKind::Full) {
                    make_frame_nullable(&mut left_frame);
                }
                if matches!(join, JoinKind::Left | JoinKind::Full) {
                    make_frame_nullable(&mut right_frame);
                }
                frame.bindings.extend(left_frame.bindings);
                frame.bindings.extend(right_frame.bindings);
                if let Some(condition) = condition {
                    let mut scopes = outer.to_vec();
                    scopes.push(frame.clone());
                    self.require_boolean(condition, &scopes)?;
                }
                for column in using_columns {
                    let matches = frame
                        .bindings
                        .iter()
                        .filter(|binding| binding.columns.contains_key(column))
                        .count();
                    if matches != 2 {
                        return Err(PostgresAnalysisError::at_start(
                            PostgresDiagnosticCode::UnknownColumn,
                            format!("JOIN USING column '{column}' must exist on both sides"),
                        ));
                    }
                    self.accessed_objects.extend(
                        frame
                            .bindings
                            .iter()
                            .filter(|binding| binding.relation.is_some())
                            .filter_map(|binding| binding.columns.get(column))
                            .map(|column| column.identity.clone()),
                    );
                }
            }
        }
        Ok(())
    }
}
