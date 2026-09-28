/// Bounded typed source grammars for the frontend coverage-guided targets.
/// Raw bytes determine structure, types, expressions, bindings, and literals.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuidedTarget {
    Lowering,
    Ownership,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Type {
    Int,
    Bool,
    Str,
    ListInt,
}

impl Type {
    fn annotation(self) -> &'static str {
        match self {
            Self::Int => "int",
            Self::Bool => "bool",
            Self::Str => "str",
            Self::ListInt => "list[int]",
        }
    }
}

#[derive(Clone)]
struct Binding {
    name: String,
    ty: Type,
}

struct Choices<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl Choices<'_> {
    fn next(&mut self) -> u8 {
        let value = if self.bytes.is_empty() {
            0
        } else {
            self.bytes[self.offset % self.bytes.len()]
        };
        self.offset += 1;
        value
    }

    fn count(&mut self) -> usize {
        4 + self.bytes.len().min(128) / 8 + usize::from(self.next() % 3)
    }

    fn binding(&mut self, bindings: &[Binding], ty: Type) -> Option<String> {
        let matching: Vec<_> = bindings.iter().filter(|item| item.ty == ty).collect();
        if matching.is_empty() {
            None
        } else {
            Some(
                matching[usize::from(self.next()) % matching.len()]
                    .name
                    .clone(),
            )
        }
    }
}

fn scalar_type(value: u8) -> Type {
    match value % 4 {
        0 => Type::Int,
        1 => Type::Bool,
        2 => Type::Str,
        _ => Type::ListInt,
    }
}

fn owned_type(value: u8) -> Type {
    if value.is_multiple_of(2) {
        Type::Str
    } else {
        Type::ListInt
    }
}

fn literal(choices: &mut Choices<'_>, ty: Type) -> String {
    match ty {
        Type::Int => format!("{}", choices.next() % 100),
        Type::Bool => {
            if choices.next().is_multiple_of(2) {
                "True".to_string()
            } else {
                "False".to_string()
            }
        }
        Type::Str => format!("\"word_{}_{}\"", choices.next() % 100, choices.next() % 100),
        Type::ListInt => format!("[{}, {}]", choices.next() % 100, choices.next() % 100),
    }
}

fn expression(
    choices: &mut Choices<'_>,
    bindings: &[Binding],
    ty: Type,
    depth: usize,
    helper_int: &str,
    helper_bool: &str,
) -> String {
    let variant = choices.next() % if depth == 0 { 2 } else { 5 };
    if variant == 0 {
        return literal(choices, ty);
    }
    if variant == 1 {
        return choices
            .binding(bindings, ty)
            .unwrap_or_else(|| literal(choices, ty));
    }
    let nested = depth - 1;
    match (ty, variant) {
        (Type::Int, 2) => format!(
            "({} + {})",
            expression(
                choices,
                bindings,
                Type::Int,
                nested,
                helper_int,
                helper_bool
            ),
            expression(
                choices,
                bindings,
                Type::Int,
                nested,
                helper_int,
                helper_bool
            ),
        ),
        (Type::Int, 3) => format!(
            "{helper_int}({}, {})",
            expression(
                choices,
                bindings,
                Type::Int,
                nested,
                helper_int,
                helper_bool
            ),
            expression(
                choices,
                bindings,
                Type::Int,
                nested,
                helper_int,
                helper_bool
            ),
        ),
        (Type::Int, _) => format!(
            "len({})",
            expression(
                choices,
                bindings,
                Type::Str,
                nested,
                helper_int,
                helper_bool
            ),
        ),
        (Type::Bool, 2) => format!(
            "({} > {})",
            expression(
                choices,
                bindings,
                Type::Int,
                nested,
                helper_int,
                helper_bool
            ),
            expression(
                choices,
                bindings,
                Type::Int,
                nested,
                helper_int,
                helper_bool
            ),
        ),
        (Type::Bool, 3) => format!(
            "({} and {})",
            expression(
                choices,
                bindings,
                Type::Bool,
                nested,
                helper_int,
                helper_bool
            ),
            expression(
                choices,
                bindings,
                Type::Bool,
                nested,
                helper_int,
                helper_bool
            ),
        ),
        (Type::Bool, _) => format!(
            "{helper_bool}({}, {})",
            expression(
                choices,
                bindings,
                Type::Int,
                nested,
                helper_int,
                helper_bool
            ),
            expression(
                choices,
                bindings,
                Type::Int,
                nested,
                helper_int,
                helper_bool
            ),
        ),
        (Type::Str, 2 | 3) => format!(
            "({} + {})",
            expression(
                choices,
                bindings,
                Type::Str,
                nested,
                helper_int,
                helper_bool
            ),
            expression(
                choices,
                bindings,
                Type::Str,
                nested,
                helper_int,
                helper_bool
            ),
        ),
        (Type::Str, _) => format!(
            "str({})",
            expression(
                choices,
                bindings,
                Type::Int,
                nested,
                helper_int,
                helper_bool
            ),
        ),
        (Type::ListInt, _) => format!(
            "[{}, {}]",
            expression(
                choices,
                bindings,
                Type::Int,
                nested,
                helper_int,
                helper_bool
            ),
            expression(
                choices,
                bindings,
                Type::Int,
                nested,
                helper_int,
                helper_bool
            ),
        ),
    }
}

fn append(source: &mut String, statement: impl AsRef<str>) {
    source.push_str(statement.as_ref());
}

struct LoweringGrammar<'a, 'b> {
    choices: &'a mut Choices<'b>,
    next_id: usize,
    helper_int: String,
    helper_bool: String,
}

impl LoweringGrammar<'_, '_> {
    fn expr(&mut self, bindings: &[Binding], ty: Type, depth: usize) -> String {
        expression(
            self.choices,
            bindings,
            ty,
            depth,
            &self.helper_int,
            &self.helper_bool,
        )
    }

    fn statement(
        &mut self,
        source: &mut String,
        bindings: &mut Vec<Binding>,
        indent: usize,
        depth: usize,
    ) {
        let id = self.next_id;
        self.next_id += 1;
        let pad = " ".repeat(indent);
        let operation = self.choices.next() % if depth == 0 { 3 } else { 6 };
        match operation {
            0 => {
                let ty = scalar_type(self.choices.next());
                let value = self.expr(bindings, ty, 2);
                let name = format!("value_{id}");
                append(
                    source,
                    format!("{pad}{name}: {} = {value}\n", ty.annotation()),
                );
                bindings.push(Binding { name, ty });
            }
            1 => {
                let binding = bindings[usize::from(self.choices.next()) % bindings.len()].clone();
                let value = self.expr(bindings, binding.ty, 2);
                append(source, format!("{pad}{} = {value}\n", binding.name));
            }
            2 => {
                let ty = scalar_type(self.choices.next());
                let value = self.expr(bindings, ty, 2);
                append(source, format!("{pad}print({value})\n"));
            }
            3 => {
                let condition = self.expr(bindings, Type::Bool, 2);
                append(source, format!("{pad}if {condition}:\n"));
                self.statement(source, &mut bindings.clone(), indent + 4, depth - 1);
                append(source, format!("{pad}else:\n"));
                self.statement(source, &mut bindings.clone(), indent + 4, depth - 1);
            }
            4 => {
                let count = self.expr(bindings, Type::Int, 1);
                let name = format!("loop_{id}");
                append(source, format!("{pad}for {name} in range({count}):\n"));
                let mut nested = bindings.clone();
                nested.push(Binding {
                    name,
                    ty: Type::Int,
                });
                self.statement(source, &mut nested, indent + 4, depth - 1);
            }
            _ => {
                let limit = self.choices.next() % 100;
                append(source, format!("{pad}while base < {limit}:\n"));
                self.statement(source, &mut bindings.clone(), indent + 4, depth - 1);
                append(
                    source,
                    format!("{}base = base + 1\n", " ".repeat(indent + 4)),
                );
            }
        }
    }
}

fn lowering_source(choices: &mut Choices<'_>) -> String {
    let helper_int = format!("add_{}", choices.next() % 100);
    let helper_bool = format!("greater_{}", choices.next() % 100);
    let mut source = format!(
        "def {helper_int}(left: int, right: int) -> int:\n    return left + right\n\n\
         def {helper_bool}(left: int, right: int) -> bool:\n    return left > right\n\n\
         def main():\n    base: int = 1\n    text: str = \"seed\"\n    items: list[int] = [1, 2]\n    flag: bool = True\n"
    );
    let count = choices.count();
    let mut grammar = LoweringGrammar {
        choices,
        next_id: 0,
        helper_int,
        helper_bool,
    };
    let mut bindings = vec![
        Binding {
            name: "base".to_string(),
            ty: Type::Int,
        },
        Binding {
            name: "text".to_string(),
            ty: Type::Str,
        },
        Binding {
            name: "items".to_string(),
            ty: Type::ListInt,
        },
        Binding {
            name: "flag".to_string(),
            ty: Type::Bool,
        },
    ];
    for _ in 0..count {
        grammar.statement(&mut source, &mut bindings, 4, 2);
    }
    source
}

struct OwnershipGrammar<'a, 'b> {
    choices: &'a mut Choices<'b>,
    next_id: usize,
    consume_str: String,
    consume_list: String,
}

impl OwnershipGrammar<'_, '_> {
    fn statement(
        &mut self,
        source: &mut String,
        bindings: &mut Vec<Binding>,
        indent: usize,
        depth: usize,
    ) {
        let id = self.next_id;
        self.next_id += 1;
        let pad = " ".repeat(indent);
        let selected = bindings[usize::from(self.choices.next()) % bindings.len()].clone();
        let operation = self.choices.next() % if depth == 0 { 5 } else { 9 };
        match operation {
            0 => {
                let ty = owned_type(self.choices.next());
                let name = format!("owned_{id}");
                let value = if self.choices.next().is_multiple_of(2) {
                    literal(self.choices, ty)
                } else {
                    self.choices
                        .binding(bindings, ty)
                        .unwrap_or_else(|| literal(self.choices, ty))
                };
                append(
                    source,
                    format!("{pad}{name}: {} = {value}\n", ty.annotation()),
                );
                bindings.push(Binding { name, ty });
            }
            1 => append(source, format!("{pad}print(len({}))\n", selected.name)),
            2 => {
                let name = format!("moved_{id}");
                append(
                    source,
                    format!(
                        "{pad}{name}: {} = {}\n",
                        selected.ty.annotation(),
                        selected.name
                    ),
                );
                bindings.push(Binding {
                    name,
                    ty: selected.ty,
                });
            }
            3 => {
                let value = literal(self.choices, selected.ty);
                append(source, format!("{pad}{} = {value}\n", selected.name));
            }
            4 => {
                let helper = if selected.ty == Type::Str {
                    &self.consume_str
                } else {
                    &self.consume_list
                };
                append(source, format!("{pad}print({helper}({}))\n", selected.name));
                if self.choices.next().is_multiple_of(2) {
                    append(source, format!("{pad}print(len({}))\n", selected.name));
                }
            }
            5 => {
                let name = format!("join_moved_{id}");
                append(
                    source,
                    format!("{pad}if len({}) > threshold:\n", selected.name),
                );
                append(
                    source,
                    format!(
                        "{}{}: {} = {}\n",
                        " ".repeat(indent + 4),
                        name,
                        selected.ty.annotation(),
                        selected.name
                    ),
                );
                self.statement(
                    source,
                    &mut bindings.clone(),
                    indent + 4,
                    depth.saturating_sub(1),
                );
                append(source, format!("{pad}else:\n"));
                append(
                    source,
                    format!("{}print(len({}))\n", " ".repeat(indent + 4), selected.name),
                );
                self.statement(
                    source,
                    &mut bindings.clone(),
                    indent + 4,
                    depth.saturating_sub(1),
                );
                append(source, format!("{pad}print(len({}))\n", selected.name));
            }
            6 => {
                let loop_name = format!("loop_{id}");
                append(
                    source,
                    format!("{pad}for {loop_name} in range(threshold):\n"),
                );
                self.statement(
                    source,
                    &mut bindings.clone(),
                    indent + 4,
                    depth.saturating_sub(1),
                );
            }
            7 => {
                let name = format!("moved_{id}");
                append(
                    source,
                    format!(
                        "{pad}{name} = {}\n{pad}print({})\n",
                        selected.name, selected.name
                    ),
                );
                bindings.push(Binding {
                    name,
                    ty: selected.ty,
                });
            }
            _ => append(source, format!("{pad}print({})\n", selected.name)),
        }
    }
}

fn ownership_source(choices: &mut Choices<'_>) -> String {
    let consume_str = format!("take_str_{}", choices.next() % 100);
    let consume_list = format!("take_list_{}", choices.next() % 100);
    let initial_str = literal(choices, Type::Str);
    let initial_list = literal(choices, Type::ListInt);
    let threshold = choices.next() % 5 + 1;
    let mut source = format!(
        "def {consume_str}(value: str) -> int:\n    return len(value)\n\n\
         def {consume_list}(value: list[int]) -> int:\n    return len(value)\n\n\
         def main():\n    threshold: int = {threshold}\n    owned_str: str = {initial_str}\n    owned_list: list[int] = {initial_list}\n"
    );
    let count = choices.count();
    let mut grammar = OwnershipGrammar {
        choices,
        next_id: 0,
        consume_str,
        consume_list,
    };
    let mut bindings = vec![
        Binding {
            name: "owned_str".to_string(),
            ty: Type::Str,
        },
        Binding {
            name: "owned_list".to_string(),
            ty: Type::ListInt,
        },
    ];
    for _ in 0..count {
        grammar.statement(&mut source, &mut bindings, 4, 2);
    }
    source
}

pub fn source_for(target: GuidedTarget, bytes: &[u8]) -> String {
    let mut choices = Choices {
        bytes: &bytes[..bytes.len().min(256)],
        offset: 0,
    };
    match target {
        GuidedTarget::Lowering => lowering_source(&mut choices),
        GuidedTarget::Ownership => ownership_source(&mut choices),
    }
}

/// Run the canonical parse and frontend HIR path. User diagnostics are valid fuzz outcomes.
/// The return value lets pinned replay tests distinguish accepted and rejected programs.
pub fn replay(target: GuidedTarget, bytes: &[u8]) -> bool {
    let source = source_for(target, bytes);
    let suite = match crate::parse_source(&source, Some("fuzz/frontend.sifr")) {
        Ok(suite) => suite,
        Err(errors) => panic!("guided grammar produced invalid syntax: {errors:?}\n{source}"),
    };
    crate::compile_module_hir_with_source(
        "main",
        &suite,
        &sifr_lowering::ExternalDefs::default(),
        crate::FrontendDiagnosticStyle::Bare,
        Some(crate::FrontendSourceContext {
            display_path: "fuzz/frontend.sifr",
            source: &source,
        }),
    )
    .is_ok()
}
