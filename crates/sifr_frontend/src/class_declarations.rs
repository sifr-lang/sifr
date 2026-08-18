//! Pre-finalization class declarations and opaque source-origin registries.

use crate::ConstValue;
use ruff_text_size::{Ranged, TextRange};
use sifr_lowering::LoweringResult;
use sifr_python_ast::{Decorator, Expr, Stmt, StmtClassDef, StmtFunctionDef};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceOriginId {
    namespace: [u8; 32],
    index: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SourceOriginKind {
    Class,
    Field,
    ClassItem,
    Method,
    Parameter,
    Annotation,
    Value,
    Decorator,
    Argument,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceOriginEntry {
    pub id: SourceOriginId,
    pub kind: SourceOriginKind,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceOriginTable {
    namespace: [u8; 32],
    entries: BTreeMap<SourceOriginId, SourceOriginEntry>,
}

impl SourceOriginTable {
    #[must_use]
    pub(crate) fn resolve(&self, id: SourceOriginId) -> Option<&SourceOriginEntry> {
        (id.namespace == self.namespace)
            .then(|| self.entries.get(&id))
            .flatten()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DecoratorDeclaration {
    name: String,
    origin: SourceOriginId,
    argument_origins: Vec<SourceOriginId>,
    arguments: Vec<CallArgumentDeclaration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CallArgumentDeclaration {
    name: Option<String>,
    origin: SourceOriginId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParameterDeclaration {
    name: String,
    origin: SourceOriginId,
    annotation_origin: Option<SourceOriginId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ClassDeclarationItem {
    Field {
        name: String,
        origin: SourceOriginId,
        annotation_origin: SourceOriginId,
        value_origin: Option<SourceOriginId>,
        value_argument_origins: Vec<SourceOriginId>,
        value_arguments: Vec<CallArgumentDeclaration>,
    },
    ClassItem {
        name: String,
        origin: SourceOriginId,
        value_origin: SourceOriginId,
        value_argument_origins: Vec<SourceOriginId>,
        value_arguments: Vec<CallArgumentDeclaration>,
    },
    Method {
        name: String,
        origin: SourceOriginId,
        decorators: Vec<DecoratorDeclaration>,
        parameters: Vec<ParameterDeclaration>,
        return_origin: Option<SourceOriginId>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClassDeclaration {
    module: String,
    name: String,
    origin: SourceOriginId,
    decorators: Vec<DecoratorDeclaration>,
    items: Vec<ClassDeclarationItem>,
    origins: SourceOriginTable,
}

impl ClassDeclaration {
    #[must_use]
    pub(crate) fn origins(&self) -> &SourceOriginTable {
        &self.origins
    }

    pub(crate) fn attach_to_shape(&self, shape: &mut ConstValue, lowering: &LoweringResult) {
        let ConstValue::Record(fields) = shape else {
            return;
        };
        fields.insert("declaration".to_string(), self.to_const_value(lowering));
    }

    pub(crate) fn to_const_value(&self, lowering: &LoweringResult) -> ConstValue {
        ConstValue::Record(BTreeMap::from([
            (
                "identity".to_string(),
                ConstValue::String(format!("{}.{}", self.module, self.name)),
            ),
            ("name".to_string(), ConstValue::String(self.name.clone())),
            ("origin".to_string(), ConstValue::SourceOrigin(self.origin)),
            (
                "decorators".to_string(),
                ConstValue::List(self.decorators.iter().map(decorator_const_value).collect()),
            ),
            (
                "items".to_string(),
                ConstValue::List(
                    self.items
                        .iter()
                        .enumerate()
                        .map(|(order, item)| self.item_const_value(order, item, lowering))
                        .collect(),
                ),
            ),
        ]))
    }

    #[must_use]
    pub(crate) fn origin_for_range(&self, range: TextRange) -> Option<SourceOriginId> {
        self.origins
            .entries
            .values()
            .find(|entry| entry.range == range)
            .or_else(|| {
                self.origins
                    .entries
                    .values()
                    .filter(|entry| entry.range.contains_range(range))
                    .min_by_key(|entry| entry.range.len())
            })
            .map(|entry| entry.id)
    }

    fn item_const_value(
        &self,
        order: usize,
        item: &ClassDeclarationItem,
        lowering: &LoweringResult,
    ) -> ConstValue {
        let item_name = match item {
            ClassDeclarationItem::Field { name, .. }
            | ClassDeclarationItem::ClassItem { name, .. }
            | ClassDeclarationItem::Method { name, .. } => name,
        };
        let mut record = BTreeMap::from([
            (
                "order".to_string(),
                ConstValue::Integer(num_bigint::BigInt::from(order)),
            ),
            (
                "identity".to_string(),
                ConstValue::String(format!("{}.{}.{}", self.module, self.name, item_name)),
            ),
            ("annotation_origin".to_string(), ConstValue::None),
            ("value_origin".to_string(), ConstValue::None),
            ("return_origin".to_string(), ConstValue::None),
            ("declared_type".to_string(), ConstValue::None),
            (
                "default_kind".to_string(),
                ConstValue::String("required".to_string()),
            ),
            ("default_value".to_string(), ConstValue::None),
            ("signature".to_string(), ConstValue::None),
            (
                "value_argument_origins".to_string(),
                ConstValue::List(Vec::new()),
            ),
            ("value_arguments".to_string(), ConstValue::List(Vec::new())),
            ("decorators".to_string(), ConstValue::List(Vec::new())),
            ("parameters".to_string(), ConstValue::List(Vec::new())),
        ]);
        match item {
            ClassDeclarationItem::Field {
                name,
                origin,
                annotation_origin,
                value_origin,
                value_argument_origins,
                value_arguments,
            } => {
                record.insert("kind".to_string(), ConstValue::String("field".to_string()));
                record.insert("name".to_string(), ConstValue::String(name.clone()));
                record.insert("origin".to_string(), ConstValue::SourceOrigin(*origin));
                record.insert(
                    "annotation_origin".to_string(),
                    ConstValue::SourceOrigin(*annotation_origin),
                );
                record.insert(
                    "value_origin".to_string(),
                    value_origin.map_or(ConstValue::None, ConstValue::SourceOrigin),
                );
                record.insert(
                    "value_argument_origins".to_string(),
                    origin_list(value_argument_origins),
                );
                record.insert(
                    "value_arguments".to_string(),
                    call_argument_list(value_arguments),
                );
                if let Some(class) = lowering
                    .module
                    .classes
                    .iter()
                    .find(|class| class.name == self.name)
                {
                    if let Some((_, ty)) = class.fields.iter().find(|(field, _)| field == name) {
                        record.insert(
                            "declared_type".to_string(),
                            ConstValue::String(crate::canonical_types::type_identity(ty)),
                        );
                    }
                    if let Some(index) = class
                        .fields
                        .iter()
                        .enumerate()
                        .rev()
                        .find_map(|(index, (field, _))| (field == name).then_some(index))
                    {
                        if let Some(value) = lowering
                            .class_field_defaults
                            .get(&self.name)
                            .into_iter()
                            .flatten()
                            .find(|(field, _)| *field == index)
                            .and_then(|(_, value)| {
                                crate::structural_shape::const_value_from_hir(value)
                            })
                        {
                            record.insert(
                                "default_kind".to_string(),
                                ConstValue::String("const".to_string()),
                            );
                            record.insert("default_value".to_string(), value);
                        }
                    }
                }
            }
            ClassDeclarationItem::ClassItem {
                name,
                origin,
                value_origin,
                value_argument_origins,
                value_arguments,
            } => {
                record.insert(
                    "kind".to_string(),
                    ConstValue::String("class_item".to_string()),
                );
                record.insert("name".to_string(), ConstValue::String(name.clone()));
                record.insert("origin".to_string(), ConstValue::SourceOrigin(*origin));
                record.insert(
                    "value_origin".to_string(),
                    ConstValue::SourceOrigin(*value_origin),
                );
                record.insert(
                    "value_argument_origins".to_string(),
                    origin_list(value_argument_origins),
                );
                record.insert(
                    "value_arguments".to_string(),
                    call_argument_list(value_arguments),
                );
            }
            ClassDeclarationItem::Method {
                name,
                origin,
                decorators,
                parameters,
                return_origin,
            } => {
                record.insert("kind".to_string(), ConstValue::String("method".to_string()));
                record.insert("name".to_string(), ConstValue::String(name.clone()));
                record.insert("origin".to_string(), ConstValue::SourceOrigin(*origin));
                record.insert(
                    "decorators".to_string(),
                    ConstValue::List(decorators.iter().map(decorator_const_value).collect()),
                );
                record.insert(
                    "parameters".to_string(),
                    ConstValue::List(parameters.iter().map(parameter_const_value).collect()),
                );
                record.insert(
                    "return_origin".to_string(),
                    return_origin.map_or(ConstValue::None, ConstValue::SourceOrigin),
                );
                let hir_name = if name == "__init__" { "new" } else { name };
                if let Some(method) = lowering
                    .module
                    .classes
                    .iter()
                    .find(|class| class.name == self.name)
                    .and_then(|class| {
                        class
                            .methods
                            .iter()
                            .chain(class.operator_impls.iter().map(|(_, method)| method))
                            .find(|method| method.name == hir_name)
                    })
                {
                    let signature = sifr_type_system::FunctionType {
                        receiver: method.receiver,
                        params: method
                            .params
                            .iter()
                            .map(|parameter| {
                                (
                                    parameter.name.clone(),
                                    parameter.ty.clone(),
                                    parameter.convention,
                                )
                            })
                            .collect(),
                        return_type: Box::new(method.return_type.clone()),
                    };
                    record.insert(
                        "signature".to_string(),
                        ConstValue::String(crate::canonical_types::function_identity(&signature)),
                    );
                }
            }
        }
        ConstValue::Record(record)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ClassDeclarationSet {
    declarations: BTreeMap<String, ClassDeclaration>,
}

impl ClassDeclarationSet {
    #[must_use]
    pub(crate) fn collect(module: &str, statements: &[Stmt]) -> Self {
        let declarations = statements
            .iter()
            .filter_map(|statement| match statement {
                Stmt::ClassDef(class) => Some((
                    class.name.to_string(),
                    DeclarationCollector::new(module, class.name.as_str()).collect(class),
                )),
                _ => None,
            })
            .collect();
        Self { declarations }
    }

    #[must_use]
    pub(crate) fn get(&self, name: &str) -> Option<&ClassDeclaration> {
        self.declarations.get(name)
    }
}

struct DeclarationCollector {
    module: String,
    class_name: String,
    namespace: [u8; 32],
    next_origin: u32,
    entries: BTreeMap<SourceOriginId, SourceOriginEntry>,
}

impl DeclarationCollector {
    fn new(module: &str, class_name: &str) -> Self {
        let namespace = sifr_structural_identity::static_program_identity(
            sifr_structural_identity::ALGORITHM_VERSION,
            [
                ("module", module.as_bytes()),
                ("class", class_name.as_bytes()),
                ("contract", b"class-declaration-origin-v1".as_slice()),
            ],
        );
        Self {
            module: module.to_string(),
            class_name: class_name.to_string(),
            namespace: *namespace.as_bytes(),
            next_origin: 0,
            entries: BTreeMap::new(),
        }
    }

    fn collect(mut self, class: &StmtClassDef) -> ClassDeclaration {
        let origin = self.add_origin(SourceOriginKind::Class, class.name.range());
        let decorators = self.decorators(&class.decorator_list);
        let items = class
            .body
            .iter()
            .filter_map(|statement| self.item(statement))
            .collect();
        ClassDeclaration {
            module: self.module,
            name: self.class_name,
            origin,
            decorators,
            items,
            origins: SourceOriginTable {
                namespace: self.namespace,
                entries: self.entries,
            },
        }
    }

    fn item(&mut self, statement: &Stmt) -> Option<ClassDeclarationItem> {
        match statement {
            Stmt::AnnAssign(assign) => {
                let Expr::Name(name) = assign.target.as_ref() else {
                    return None;
                };
                let origin = self.add_origin(SourceOriginKind::Field, name.range());
                let annotation_origin =
                    self.add_origin(SourceOriginKind::Annotation, assign.annotation.range());
                let value_origin = assign
                    .value
                    .as_deref()
                    .map(|value| self.add_origin(SourceOriginKind::Value, value.range()));
                let value_arguments = assign
                    .value
                    .as_deref()
                    .map_or_else(Vec::new, |value| self.call_arguments(value));
                let value_argument_origins = argument_origins(&value_arguments);
                Some(ClassDeclarationItem::Field {
                    name: name.id.to_string(),
                    origin,
                    annotation_origin,
                    value_origin,
                    value_argument_origins,
                    value_arguments,
                })
            }
            Stmt::Assign(assign) if assign.targets.len() == 1 => {
                let Expr::Name(name) = &assign.targets[0] else {
                    return None;
                };
                let origin = self.add_origin(SourceOriginKind::ClassItem, name.range());
                let value_origin = self.add_origin(SourceOriginKind::Value, assign.value.range());
                let value_arguments = self.call_arguments(&assign.value);
                let value_argument_origins = argument_origins(&value_arguments);
                Some(ClassDeclarationItem::ClassItem {
                    name: name.id.to_string(),
                    origin,
                    value_origin,
                    value_argument_origins,
                    value_arguments,
                })
            }
            Stmt::FunctionDef(function) => Some(self.method(function)),
            _ => None,
        }
    }

    fn method(&mut self, function: &StmtFunctionDef) -> ClassDeclarationItem {
        let origin = self.add_origin(SourceOriginKind::Method, function.name.range());
        let decorators = self.decorators(&function.decorator_list);
        let parameters = function
            .parameters
            .posonlyargs
            .iter()
            .chain(&function.parameters.args)
            .chain(&function.parameters.kwonlyargs)
            .map(|parameter| {
                let origin = self.add_origin(
                    SourceOriginKind::Parameter,
                    parameter.parameter.name.range(),
                );
                let annotation_origin =
                    parameter.parameter.annotation.as_deref().map(|annotation| {
                        self.add_origin(SourceOriginKind::Annotation, annotation.range())
                    });
                ParameterDeclaration {
                    name: parameter.parameter.name.to_string(),
                    origin,
                    annotation_origin,
                }
            })
            .collect();
        let return_origin = function
            .returns
            .as_deref()
            .map(|returns| self.add_origin(SourceOriginKind::Annotation, returns.range()));
        ClassDeclarationItem::Method {
            name: function.name.to_string(),
            origin,
            decorators,
            parameters,
            return_origin,
        }
    }

    fn decorators(&mut self, decorators: &[Decorator]) -> Vec<DecoratorDeclaration> {
        decorators
            .iter()
            .map(|decorator| {
                let origin =
                    self.add_origin(SourceOriginKind::Decorator, decorator.expression.range());
                let arguments = self.call_arguments(&decorator.expression);
                DecoratorDeclaration {
                    name: expression_name(&decorator.expression),
                    origin,
                    argument_origins: argument_origins(&arguments),
                    arguments,
                }
            })
            .collect()
    }

    fn call_arguments(&mut self, expression: &Expr) -> Vec<CallArgumentDeclaration> {
        let Expr::Call(call) = expression else {
            return Vec::new();
        };
        let mut arguments = call
            .arguments
            .args
            .iter()
            .map(|argument| CallArgumentDeclaration {
                name: None,
                origin: self.add_origin(SourceOriginKind::Argument, argument.range()),
            })
            .collect::<Vec<_>>();
        arguments.extend(
            call.arguments
                .keywords
                .iter()
                .map(|keyword| CallArgumentDeclaration {
                    name: keyword.arg.as_ref().map(ToString::to_string),
                    origin: self.add_origin(SourceOriginKind::Argument, keyword.value.range()),
                }),
        );
        arguments
    }

    fn add_origin(&mut self, kind: SourceOriginKind, range: TextRange) -> SourceOriginId {
        let id = SourceOriginId {
            namespace: self.namespace,
            index: self.next_origin,
        };
        self.next_origin = self.next_origin.saturating_add(1);
        self.entries
            .insert(id, SourceOriginEntry { id, kind, range });
        id
    }
}

fn origin_list(origins: &[SourceOriginId]) -> ConstValue {
    ConstValue::List(
        origins
            .iter()
            .copied()
            .map(ConstValue::SourceOrigin)
            .collect(),
    )
}

fn argument_origins(arguments: &[CallArgumentDeclaration]) -> Vec<SourceOriginId> {
    arguments.iter().map(|argument| argument.origin).collect()
}

fn call_argument_list(arguments: &[CallArgumentDeclaration]) -> ConstValue {
    ConstValue::List(
        arguments
            .iter()
            .map(|argument| {
                ConstValue::Record(BTreeMap::from([
                    (
                        "name".to_string(),
                        argument
                            .name
                            .as_ref()
                            .map_or(ConstValue::None, |name| ConstValue::String(name.clone())),
                    ),
                    (
                        "origin".to_string(),
                        ConstValue::SourceOrigin(argument.origin),
                    ),
                ]))
            })
            .collect(),
    )
}

fn decorator_const_value(decorator: &DecoratorDeclaration) -> ConstValue {
    ConstValue::Record(BTreeMap::from([
        (
            "name".to_string(),
            ConstValue::String(decorator.name.clone()),
        ),
        (
            "origin".to_string(),
            ConstValue::SourceOrigin(decorator.origin),
        ),
        (
            "argument_origins".to_string(),
            origin_list(&decorator.argument_origins),
        ),
        (
            "arguments".to_string(),
            call_argument_list(&decorator.arguments),
        ),
    ]))
}

fn parameter_const_value(parameter: &ParameterDeclaration) -> ConstValue {
    ConstValue::Record(BTreeMap::from([
        (
            "name".to_string(),
            ConstValue::String(parameter.name.clone()),
        ),
        (
            "origin".to_string(),
            ConstValue::SourceOrigin(parameter.origin),
        ),
        (
            "annotation_origin".to_string(),
            parameter
                .annotation_origin
                .map_or(ConstValue::None, ConstValue::SourceOrigin),
        ),
    ]))
}

fn expression_name(expression: &Expr) -> String {
    match expression {
        Expr::Name(name) => name.id.to_string(),
        Expr::Attribute(attribute) => format!(
            "{}.{}",
            expression_name(&attribute.value),
            attribute.attr.as_str()
        ),
        Expr::Call(call) => expression_name(&call.func),
        _ => "<expression>".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_syntax::parse_module_suite;

    #[test]
    fn declaration_order_and_origin_ids_ignore_source_movement() {
        let source = r#"
@metadata("fixture.class", "contract")
class Command:
    payload: str = factory("value")
    config = settings("strict")

    @metadata("fixture.method", "decode")
    def decode(self, value: str) -> str:
        return value
"#;
        let moved = format!("\n\n{source}");
        let first = parse_module_suite(source, None).expect("fixture parses");
        let second = parse_module_suite(&moved, None).expect("moved fixture parses");
        let first = ClassDeclarationSet::collect("fixture", &first);
        let second = ClassDeclarationSet::collect("fixture", &second);
        let first = first.get("Command").expect("declaration exists");
        let second = second.get("Command").expect("declaration exists");

        assert_eq!(first.items.len(), 3);
        assert_eq!(first.origin, second.origin);
        assert_ne!(
            first.origins.resolve(first.origin).map(|entry| entry.range),
            second
                .origins
                .resolve(second.origin)
                .map(|entry| entry.range)
        );
    }

    #[test]
    fn origin_registry_rejects_another_class_namespace() {
        let parsed = parse_module_suite("class A:\n    x: int\nclass B:\n    y: int\n", None)
            .expect("fixture parses");
        let declarations = ClassDeclarationSet::collect("fixture", &parsed);
        let a = declarations.get("A").expect("A exists");
        let b = declarations.get("B").expect("B exists");
        assert!(a.origins.resolve(a.origin).is_some());
        assert!(a.origins.resolve(b.origin).is_none());
    }

    #[test]
    fn call_argument_origins_preserve_keyword_names_and_source_order() {
        let parsed = parse_module_suite(
            "@decorate(1, mode=\"strict\")\nclass Model:\n    value: int = field(2, alias=\"id\")\n",
            None,
        )
        .expect("fixture parses");
        let declarations = ClassDeclarationSet::collect("fixture", &parsed);
        let model = declarations.get("Model").expect("Model exists");
        assert_eq!(
            model.decorators[0]
                .arguments
                .iter()
                .map(|argument| argument.name.as_deref())
                .collect::<Vec<_>>(),
            vec![None, Some("mode")],
        );
        let ClassDeclarationItem::Field {
            value_arguments, ..
        } = &model.items[0]
        else {
            panic!("field declaration should be retained");
        };
        assert_eq!(
            value_arguments
                .iter()
                .map(|argument| argument.name.as_deref())
                .collect::<Vec<_>>(),
            vec![None, Some("alias")],
        );
    }
}
