use sifr_python_ast::{Expr, Stmt};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::lower) enum AsyncSuspensionSummary {
    NoSuspend,
    Suspends,
}

impl AsyncSuspensionSummary {
    fn union(self, other: Self) -> Self {
        if matches!(self, Self::Suspends) || matches!(other, Self::Suspends) {
            Self::Suspends
        } else {
            Self::NoSuspend
        }
    }
}

pub(in crate::lower) fn collect_async_suspension_summaries(
    stmts: &[Stmt],
) -> HashMap<String, AsyncSuspensionSummary> {
    let async_functions = collect_top_level_async_functions(stmts);
    let mut summaries = async_functions
        .iter()
        .map(|name| (name.clone(), AsyncSuspensionSummary::NoSuspend))
        .collect::<HashMap<_, _>>();

    let mut changed = true;
    while changed {
        changed = false;
        for stmt in stmts {
            let Stmt::FunctionDef(func) = stmt else {
                continue;
            };
            let name = func.name.to_string();
            if !async_functions.contains(&name) {
                continue;
            }
            let next = summarize_stmts(&func.body, &async_functions, &summaries);
            if summaries.get(&name).copied() != Some(next) {
                summaries.insert(name, next);
                changed = true;
            }
        }
    }

    summaries
}

fn collect_top_level_async_functions(stmts: &[Stmt]) -> HashSet<String> {
    stmts
        .iter()
        .filter_map(|stmt| {
            let Stmt::FunctionDef(func) = stmt else {
                return None;
            };
            func.is_async.then(|| func.name.to_string())
        })
        .collect()
}

fn summarize_stmts(
    stmts: &[Stmt],
    async_functions: &HashSet<String>,
    summaries: &HashMap<String, AsyncSuspensionSummary>,
) -> AsyncSuspensionSummary {
    stmts
        .iter()
        .fold(AsyncSuspensionSummary::NoSuspend, |acc, stmt| {
            acc.union(summarize_stmt(stmt, async_functions, summaries))
        })
}

fn summarize_stmt(
    stmt: &Stmt,
    async_functions: &HashSet<String>,
    summaries: &HashMap<String, AsyncSuspensionSummary>,
) -> AsyncSuspensionSummary {
    match stmt {
        Stmt::Expr(expr_stmt) => {
            summarize_expr(expr_stmt.value.as_ref(), async_functions, summaries)
        }
        Stmt::Return(ret) => ret
            .value
            .as_deref()
            .map_or(AsyncSuspensionSummary::NoSuspend, |expr| {
                summarize_expr(expr, async_functions, summaries)
            }),
        Stmt::AnnAssign(ann) => ann
            .value
            .as_deref()
            .map_or(AsyncSuspensionSummary::NoSuspend, |expr| {
                summarize_expr(expr, async_functions, summaries)
            }),
        Stmt::Assign(assign) => summarize_expr(assign.value.as_ref(), async_functions, summaries),
        Stmt::AugAssign(aug) => summarize_expr(aug.value.as_ref(), async_functions, summaries),
        Stmt::If(if_stmt) => {
            let mut summary = summarize_expr(if_stmt.test.as_ref(), async_functions, summaries)
                .union(summarize_stmts(&if_stmt.body, async_functions, summaries));
            for clause in &if_stmt.elif_else_clauses {
                summary = summary.union(
                    clause
                        .test
                        .as_ref()
                        .map_or(AsyncSuspensionSummary::NoSuspend, |test| {
                            summarize_expr(test, async_functions, summaries)
                        }),
                );
                summary = summary.union(summarize_stmts(&clause.body, async_functions, summaries));
            }
            summary
        }
        Stmt::While(while_stmt) => {
            summarize_expr(while_stmt.test.as_ref(), async_functions, summaries)
                .union(summarize_stmts(
                    &while_stmt.body,
                    async_functions,
                    summaries,
                ))
                .union(summarize_stmts(
                    &while_stmt.orelse,
                    async_functions,
                    summaries,
                ))
        }
        Stmt::For(for_stmt) => {
            let base = summarize_expr(for_stmt.iter.as_ref(), async_functions, summaries)
                .union(summarize_stmts(&for_stmt.body, async_functions, summaries))
                .union(summarize_stmts(
                    &for_stmt.orelse,
                    async_functions,
                    summaries,
                ));
            if for_stmt.is_async {
                base.union(AsyncSuspensionSummary::Suspends)
            } else {
                base
            }
        }
        Stmt::With(with_stmt) => {
            let base = with_stmt
                .items
                .iter()
                .fold(AsyncSuspensionSummary::NoSuspend, |acc, item| {
                    acc.union(summarize_expr(
                        &item.context_expr,
                        async_functions,
                        summaries,
                    ))
                })
                .union(summarize_stmts(&with_stmt.body, async_functions, summaries));
            if with_stmt.is_async {
                base.union(AsyncSuspensionSummary::Suspends)
            } else {
                base
            }
        }
        Stmt::Try(try_stmt) => {
            let mut summary = summarize_stmts(&try_stmt.body, async_functions, summaries);
            for handler in &try_stmt.handlers {
                let sifr_python_ast::ExceptHandler::ExceptHandler(handler) = handler;
                summary = summary.union(summarize_stmts(&handler.body, async_functions, summaries));
            }
            summary
                .union(summarize_stmts(
                    &try_stmt.orelse,
                    async_functions,
                    summaries,
                ))
                .union(summarize_stmts(
                    &try_stmt.finalbody,
                    async_functions,
                    summaries,
                ))
        }
        Stmt::FunctionDef(_) | Stmt::ClassDef(_) => AsyncSuspensionSummary::NoSuspend,
        _ => AsyncSuspensionSummary::NoSuspend,
    }
}

fn summarize_expr(
    expr: &Expr,
    async_functions: &HashSet<String>,
    summaries: &HashMap<String, AsyncSuspensionSummary>,
) -> AsyncSuspensionSummary {
    match expr {
        Expr::Await(await_expr) => {
            summarize_awaited_expr(await_expr.value.as_ref(), async_functions, summaries)
        }
        Expr::Yield(_) | Expr::YieldFrom(_) => AsyncSuspensionSummary::Suspends,
        Expr::Call(call) => {
            let mut summary = summarize_expr(call.func.as_ref(), async_functions, summaries);
            for arg in &call.arguments.args {
                summary = summary.union(summarize_expr(arg, async_functions, summaries));
            }
            for keyword in &call.arguments.keywords {
                summary = summary.union(summarize_expr(&keyword.value, async_functions, summaries));
            }
            summary
        }
        Expr::Attribute(attr) => summarize_expr(attr.value.as_ref(), async_functions, summaries),
        Expr::Subscript(sub) => summarize_expr(sub.value.as_ref(), async_functions, summaries)
            .union(summarize_expr(
                sub.slice.as_ref(),
                async_functions,
                summaries,
            )),
        Expr::BinOp(bin) => summarize_expr(bin.left.as_ref(), async_functions, summaries).union(
            summarize_expr(bin.right.as_ref(), async_functions, summaries),
        ),
        Expr::BoolOp(bool_op) => bool_op
            .values
            .iter()
            .fold(AsyncSuspensionSummary::NoSuspend, |acc, value| {
                acc.union(summarize_expr(value, async_functions, summaries))
            }),
        Expr::UnaryOp(unary) => summarize_expr(unary.operand.as_ref(), async_functions, summaries),
        Expr::Compare(compare) => {
            let mut summary = summarize_expr(compare.left.as_ref(), async_functions, summaries);
            for comparator in &compare.comparators {
                summary = summary.union(summarize_expr(comparator, async_functions, summaries));
            }
            summary
        }
        Expr::If(if_expr) => summarize_expr(if_expr.test.as_ref(), async_functions, summaries)
            .union(summarize_expr(
                if_expr.body.as_ref(),
                async_functions,
                summaries,
            ))
            .union(summarize_expr(
                if_expr.orelse.as_ref(),
                async_functions,
                summaries,
            )),
        Expr::List(list) => list
            .elts
            .iter()
            .fold(AsyncSuspensionSummary::NoSuspend, |acc, value| {
                acc.union(summarize_expr(value, async_functions, summaries))
            }),
        Expr::Tuple(tuple) => tuple
            .elts
            .iter()
            .fold(AsyncSuspensionSummary::NoSuspend, |acc, value| {
                acc.union(summarize_expr(value, async_functions, summaries))
            }),
        Expr::Set(set) => set
            .elts
            .iter()
            .fold(AsyncSuspensionSummary::NoSuspend, |acc, value| {
                acc.union(summarize_expr(value, async_functions, summaries))
            }),
        Expr::Dict(dict) => {
            dict.items
                .iter()
                .fold(AsyncSuspensionSummary::NoSuspend, |acc, item| {
                    let key_summary = item
                        .key
                        .as_ref()
                        .map_or(AsyncSuspensionSummary::NoSuspend, |key| {
                            summarize_expr(key, async_functions, summaries)
                        });
                    acc.union(key_summary).union(summarize_expr(
                        &item.value,
                        async_functions,
                        summaries,
                    ))
                })
        }
        Expr::ListComp(comp) => summarize_comprehension(
            &comp.generators,
            Some(comp.elt.as_ref()),
            None,
            async_functions,
            summaries,
        ),
        Expr::SetComp(comp) => summarize_comprehension(
            &comp.generators,
            Some(comp.elt.as_ref()),
            None,
            async_functions,
            summaries,
        ),
        Expr::DictComp(comp) => summarize_comprehension(
            &comp.generators,
            Some(comp.key.as_ref()),
            Some(comp.value.as_ref()),
            async_functions,
            summaries,
        ),
        _ => AsyncSuspensionSummary::NoSuspend,
    }
}

fn summarize_awaited_expr(
    expr: &Expr,
    async_functions: &HashSet<String>,
    summaries: &HashMap<String, AsyncSuspensionSummary>,
) -> AsyncSuspensionSummary {
    if let Expr::Call(call) = expr {
        if let Expr::Name(name) = call.func.as_ref() {
            let function_name = name.id.to_string();
            if async_functions.contains(&function_name) {
                return summaries
                    .get(&function_name)
                    .copied()
                    .unwrap_or(AsyncSuspensionSummary::NoSuspend);
            }
        }
    }

    AsyncSuspensionSummary::Suspends.union(summarize_expr(expr, async_functions, summaries))
}

fn summarize_comprehension(
    generators: &[sifr_python_ast::Comprehension],
    first_expr: Option<&Expr>,
    second_expr: Option<&Expr>,
    async_functions: &HashSet<String>,
    summaries: &HashMap<String, AsyncSuspensionSummary>,
) -> AsyncSuspensionSummary {
    let mut summary = AsyncSuspensionSummary::NoSuspend;
    if generators.iter().any(|generator| generator.is_async) {
        summary = summary.union(AsyncSuspensionSummary::Suspends);
    }
    if let Some(expr) = first_expr {
        summary = summary.union(summarize_expr(expr, async_functions, summaries));
    }
    if let Some(expr) = second_expr {
        summary = summary.union(summarize_expr(expr, async_functions, summaries));
    }
    for generator in generators {
        summary =
            summary
                .union(summarize_expr(&generator.iter, async_functions, summaries))
                .union(generator.ifs.iter().fold(
                    AsyncSuspensionSummary::NoSuspend,
                    |acc, condition| {
                        acc.union(summarize_expr(condition, async_functions, summaries))
                    },
                ));
    }
    summary
}

#[cfg(test)]
mod tests {
    use super::{collect_async_suspension_summaries, AsyncSuspensionSummary};
    use sifr_python_parser::parse_module;

    fn summaries(source: &str) -> std::collections::HashMap<String, AsyncSuspensionSummary> {
        let parsed = parse_module(source).expect("parse failed");
        collect_async_suspension_summaries(&parsed.into_suite())
    }

    #[test]
    fn marks_direct_timer_wait_as_suspending() {
        let summaries = summaries(
            r"
async def main() -> None:
    await task.sleep(0.0)
",
        );
        assert_eq!(
            summaries.get("main"),
            Some(&AsyncSuspensionSummary::Suspends)
        );
    }

    #[test]
    fn propagates_transitive_same_task_await_summaries() {
        let summaries = summaries(
            r"
async def leaf() -> int:
    await task.sleep(0.0)
    return 1

async def wrapper() -> int:
    return await leaf()
",
        );
        assert_eq!(
            summaries.get("wrapper"),
            Some(&AsyncSuspensionSummary::Suspends)
        );
    }

    #[test]
    fn keeps_fake_async_wrapper_chain_no_suspend() {
        let summaries = summaries(
            r"
async def leaf() -> int:
    return 1

async def wrapper() -> int:
    return await leaf()
",
        );
        assert_eq!(
            summaries.get("leaf"),
            Some(&AsyncSuspensionSummary::NoSuspend)
        );
        assert_eq!(
            summaries.get("wrapper"),
            Some(&AsyncSuspensionSummary::NoSuspend)
        );
    }

    #[test]
    fn marks_async_generator_yield_as_suspending() {
        let summaries = summaries(
            r"
async def values() -> AsyncGenerator[int, GeneratorCloseError]:
    yield 1
",
        );
        assert_eq!(
            summaries.get("values"),
            Some(&AsyncSuspensionSummary::Suspends)
        );
    }
}
