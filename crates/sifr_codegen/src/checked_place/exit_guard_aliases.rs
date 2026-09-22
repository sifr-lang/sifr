use std::collections::{HashMap, HashSet};

use crate::{HirExpr, HirStmt, RustEmitter, Type};

#[derive(Clone)]
enum IndexAlias {
    Literal(i64),
    Last(String),
}

fn alias_value(value: &HirExpr, aliases: &HashMap<String, IndexAlias>) -> Option<IndexAlias> {
    match value {
        HirExpr::IntLiteral(value) => Some(IndexAlias::Literal(*value)),
        HirExpr::Name { name, .. } => aliases.get(name).cloned(),
        HirExpr::BinOp {
            left, op, right, ..
        } if op == "-" && matches!(right.as_ref(), HirExpr::IntLiteral(1)) => {
            let HirExpr::MethodCall {
                object,
                method,
                args,
                ..
            } = left.as_ref()
            else {
                return None;
            };
            let HirExpr::Name { name, ty, .. } = object.as_ref() else {
                return None;
            };
            (method == "len"
                && args.is_empty()
                && matches!(ty.resolve_alias(), Type::List(_) | Type::Str | Type::Bytes))
            .then(|| IndexAlias::Last(name.clone()))
        }
        _ => None,
    }
}

impl RustEmitter {
    /// Preserve endpoint proofs through a pure declaration prefix. No effect,
    /// rebinding, or control-flow boundary may be crossed by these early reads.
    pub(super) fn aliased_exit_guard_reads(
        &self,
        following: &[HirStmt],
    ) -> Vec<(HirExpr, HirExpr)> {
        let mut aliases = HashMap::new();
        let mut introduced = HashSet::new();
        let mut reads = Vec::new();
        for stmt in following {
            let bindings = match stmt {
                HirStmt::Let { name, value, .. } => vec![(name.as_str(), value)],
                HirStmt::TupleUnpack {
                    targets,
                    value: HirExpr::TupleLiteral { elements, .. },
                } if targets.len() == elements.len() => {
                    let Some(bindings) = targets
                        .iter()
                        .zip(elements)
                        .map(|(target, value)| match &target.binding {
                            sifr_ir::HirTupleTargetBinding::Name(name)
                                if !target.rebind_existing =>
                            {
                                Some((name.as_str(), value))
                            }
                            _ => None,
                        })
                        .collect::<Option<Vec<_>>>()
                    else {
                        break;
                    };
                    bindings
                }
                _ => break,
            };
            let proven = self.body_analysis.atomic_proven_reads_in(stmt);
            let mut additions = Vec::new();
            let mut statement_reads = Vec::new();
            let mut names = HashSet::new();
            let safe = bindings.into_iter().all(|(name, value)| {
                if introduced.contains(name) || !names.insert(name.to_string()) {
                    return false;
                }
                if let Some(alias) = alias_value(value, &aliases) {
                    if matches!(&alias, IndexAlias::Last(object) if introduced.contains(object)) {
                        return false;
                    }
                    additions.push((name.to_string(), alias));
                    return true;
                }
                let HirExpr::Index { object, index, ty } = value else {
                    return false;
                };
                let HirExpr::Name {
                    name: object_name, ..
                } = object.as_ref()
                else {
                    return false;
                };
                if introduced.contains(object_name)
                    || !matches!(
                        object.ty().resolve_alias(),
                        Type::List(_) | Type::Str | Type::Bytes
                    )
                    || !(crate::helpers::is_copy_type_for_codegen(ty)
                        || matches!(ty.resolve_alias(), Type::Int | Type::Str | Type::Bytes))
                {
                    return false;
                }
                let key = super::checked_place_read_key(object, index);
                if key.is_none()
                    || !proven.iter().any(|read| {
                        matches!(read, HirExpr::Index { object, index, .. }
                        if super::checked_place_read_key(object, index) == key)
                    })
                {
                    return false;
                }
                let index = match alias_value(index, &aliases) {
                    Some(IndexAlias::Literal(value)) => value,
                    Some(IndexAlias::Last(name)) if name == *object_name => -1,
                    _ => return false,
                };
                statement_reads.push((
                    value.clone(),
                    HirExpr::Index {
                        object: object.clone(),
                        index: Box::new(HirExpr::IntLiteral(index)),
                        ty: ty.clone(),
                    },
                ));
                true
            });
            if !safe {
                break;
            }
            introduced.extend(names);
            aliases.extend(additions);
            reads.extend(statement_reads);
        }
        reads
    }
}
