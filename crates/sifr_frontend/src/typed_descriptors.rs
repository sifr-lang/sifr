//! Typed collection and bounded evaluation of package declaration descriptors.

use crate::{ConstValue, DeterministicConstEvaluator};
use num_bigint::BigInt;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_lowering::{
    DeclarationDescriptorFunction, DeclarationDescriptorKind, ExternalDefs, HirDiagnostic,
    HirFunction, HirModule, LoweringResult, StaticProgramValue, TypedDeclarationDescriptor,
};
use sifr_python_ast::visitor::{self, Visitor};
use sifr_python_ast::{Expr, ExprCall, Number, Operator, Stmt, StmtClassDef};
use sifr_type_system::{FunctionType, Type};
use std::collections::{BTreeMap, HashMap, HashSet};

type DescriptorResult<T> = Result<T, Box<HirDiagnostic>>;

mod nested_const_calls;

pub(crate) fn collect(
    module_name: &str,
    statements: &[Stmt],
    result: &mut LoweringResult,
    external_defs: &ExternalDefs,
) -> Result<(), Vec<HirDiagnostic>> {
    let resolver = DescriptorResolver::new(module_name, &result.module, result, external_defs);
    if resolver.bindings.is_empty() {
        return Ok(());
    }
    let mut collector = DescriptorCollector {
        module_name,
        result,
        external_defs,
        resolver: &resolver,
        accepted_calls: HashSet::new(),
        descriptors: Vec::new(),
        current_provider: None,
        errors: Vec::new(),
    };
    for statement in statements {
        if let Stmt::ClassDef(class) = statement {
            collector.collect_class(class);
        }
    }
    let mut invalid = InvalidLocationVisitor {
        resolver: &resolver,
        accepted: &collector.accepted_calls,
        errors: Vec::new(),
    };
    for statement in statements {
        invalid.visit_stmt(statement);
    }
    collector.errors.extend(invalid.errors);
    if collector.errors.is_empty() {
        collector.result.declaration_descriptors = collector.descriptors;
        Ok(())
    } else {
        Err(collector.errors)
    }
}

struct DescriptorResolver {
    bindings: HashMap<String, DeclarationDescriptorFunction>,
}

impl DescriptorResolver {
    fn new(
        module_name: &str,
        module: &HirModule,
        result: &LoweringResult,
        external_defs: &ExternalDefs,
    ) -> Self {
        let mut bindings = result
            .descriptor_functions
            .iter()
            .map(|declaration| (declaration.function.clone(), declaration.clone()))
            .collect::<HashMap<_, _>>();
        for import in &module.imports {
            let Some(exports) = external_defs.descriptor_functions.get(&import.module) else {
                continue;
            };
            for name in &import.names {
                let Some(declaration) = exports.get(name) else {
                    continue;
                };
                let local = import
                    .aliases
                    .iter()
                    .find(|(original, _)| original == name)
                    .map_or_else(|| name.clone(), |(_, alias)| alias.clone());
                bindings.insert(local, declaration.clone());
            }
        }
        bindings.retain(|_, declaration| {
            declaration.module == module_name
                || external_defs
                    .descriptor_functions
                    .get(&declaration.module)
                    .is_some_and(|exports| exports.contains_key(&declaration.function))
        });
        Self { bindings }
    }

    fn call<'a>(
        &'a self,
        expression: &'a Expr,
    ) -> Option<(&'a DeclarationDescriptorFunction, &'a ExprCall)> {
        let Expr::Call(call) = expression else {
            return None;
        };
        let Expr::Name(name) = call.func.as_ref() else {
            return None;
        };
        Some((self.bindings.get(name.id.as_str())?, call))
    }
}

struct DescriptorCollector<'a> {
    module_name: &'a str,
    result: &'a mut LoweringResult,
    external_defs: &'a ExternalDefs,
    resolver: &'a DescriptorResolver,
    accepted_calls: HashSet<TextRange>,
    descriptors: Vec<TypedDeclarationDescriptor>,
    current_provider: Option<(String, String)>,
    errors: Vec<HirDiagnostic>,
}

impl DescriptorCollector<'_> {
    fn collect_class(&mut self, class: &StmtClassDef) {
        self.current_provider = None;
        let owner = class.name.to_string();
        for statement in &class.body {
            match statement {
                Stmt::AnnAssign(assignment) => {
                    let target = match assignment.target.as_ref() {
                        Expr::Name(name) => name.id.to_string(),
                        _ => continue,
                    };
                    self.collect_annotation(
                        &assignment.annotation,
                        &owner,
                        format!("{owner}.{target}:type"),
                    );
                    if let Some(value) = &assignment.value {
                        self.collect_use(
                            value,
                            DeclarationDescriptorKind::Field,
                            &owner,
                            format!("{owner}.{target}"),
                        );
                    }
                }
                Stmt::Assign(assignment) => {
                    let target = assignment
                        .targets
                        .first()
                        .and_then(|target| match target {
                            Expr::Name(name) => Some(name.id.to_string()),
                            _ => None,
                        })
                        .unwrap_or_else(|| "<class-item>".to_string());
                    self.collect_use(
                        &assignment.value,
                        DeclarationDescriptorKind::Class,
                        &owner,
                        format!("{owner}.{target}"),
                    );
                }
                Stmt::FunctionDef(function) => {
                    let method = function.name.to_string();
                    let hir_name = if method == "__init__" { "new" } else { &method };
                    let method_kind = self
                        .result
                        .module
                        .classes
                        .iter()
                        .find(|candidate| candidate.name == owner)
                        .and_then(|candidate| {
                            candidate
                                .methods
                                .iter()
                                .chain(candidate.operator_impls.iter().map(|(_, method)| method))
                                .find(|candidate| candidate.name == hir_name)
                        })
                        .map_or("regular", |candidate| match candidate.method_kind {
                            sifr_lowering::MethodKind::Regular => "regular",
                            sifr_lowering::MethodKind::StaticMethod => "static",
                            sifr_lowering::MethodKind::ClassMethod => "class",
                        });
                    let classmethod_index = function.decorator_list.iter().position(|decorator| {
                        matches!(
                            &decorator.expression,
                            Expr::Name(name) if name.id.as_str() == "classmethod"
                        )
                    });
                    if let Some(classmethod_index) = classmethod_index {
                        for (index, decorator) in function.decorator_list.iter().enumerate() {
                            let Some((declaration, call)) =
                                self.resolver.call(&decorator.expression)
                            else {
                                continue;
                            };
                            if declaration.kind == DeclarationDescriptorKind::Method
                                && (index + 1 != classmethod_index
                                    || classmethod_index + 1 != function.decorator_list.len())
                            {
                                self.errors.push(diagnostic(
                                    DiagnosticCode::META_MALFORMED_DECLARATION,
                                    format!(
                                        "method descriptor '{}' must be the outer decorator with @classmethod directly above the method",
                                        declaration.function
                                    ),
                                    call.range(),
                                ));
                            }
                        }
                    }
                    for decorator in &function.decorator_list {
                        self.collect_use(
                            &decorator.expression,
                            DeclarationDescriptorKind::Method,
                            &owner,
                            format!("{owner}.{method}:{method_kind}"),
                        );
                    }
                    for parameter in function
                        .parameters
                        .posonlyargs
                        .iter()
                        .chain(&function.parameters.args)
                    {
                        if let Some(annotation) = &parameter.parameter.annotation {
                            self.collect_annotation(
                                annotation,
                                &owner,
                                format!("{owner}.{method}:parameter:{}", parameter.parameter.name),
                            );
                        }
                    }
                    for parameter in &function.parameters.kwonlyargs {
                        if let Some(annotation) = &parameter.parameter.annotation {
                            self.collect_annotation(
                                annotation,
                                &owner,
                                format!("{owner}.{method}:parameter:{}", parameter.parameter.name),
                            );
                        }
                    }
                    if let Some(annotation) = &function.returns {
                        self.collect_annotation(
                            annotation,
                            &owner,
                            format!("{owner}.{method}:return"),
                        );
                    }
                }
                _ => {}
            }
        }
    }

    fn collect_annotation(&mut self, expression: &Expr, owner: &str, target: String) {
        if let Expr::BinOp(union) = expression {
            if matches!(union.op, Operator::BitOr) {
                self.collect_annotation(&union.left, owner, target.clone());
                self.collect_annotation(&union.right, owner, target);
            }
            return;
        }
        let Expr::Subscript(subscript) = expression else {
            return;
        };
        if matches!(subscript.value.as_ref(), Expr::Name(name) if name.id.as_str() == "Annotated") {
            let Expr::Tuple(tuple) = subscript.slice.as_ref() else {
                return;
            };
            // Flatten from the declared type outward. This preserves the
            // source order of nested Annotated layers, after which RHS
            // descriptors are collected by the surrounding declaration pass.
            if let Some(inner) = tuple.elts.first() {
                self.collect_annotation(inner, owner, target.clone());
            }
            for descriptor in tuple.elts.iter().skip(1) {
                self.collect_use(
                    descriptor,
                    DeclarationDescriptorKind::Type,
                    owner,
                    target.clone(),
                );
            }
            return;
        }
        match subscript.slice.as_ref() {
            Expr::Tuple(tuple) => {
                for item in &tuple.elts {
                    self.collect_annotation(item, owner, target.clone());
                }
            }
            item => self.collect_annotation(item, owner, target),
        }
    }

    fn collect_use(
        &mut self,
        expression: &Expr,
        expected_kind: DeclarationDescriptorKind,
        owner: &str,
        target_identity: String,
    ) {
        let Some((declaration, call)) = self.resolver.call(expression) else {
            return;
        };
        self.accepted_calls.insert(call.range());
        if declaration.kind != expected_kind {
            self.errors.push(diagnostic(
                DiagnosticCode::META_MALFORMED_DECLARATION,
                format!(
                    "{} descriptor '{}' is not valid in a {} descriptor location",
                    descriptor_kind_name(declaration.kind),
                    declaration.function,
                    descriptor_kind_name(expected_kind),
                ),
                call.range(),
            ));
            return;
        }
        let provider = (
            declaration.provider_module.clone(),
            declaration.provider_function.clone(),
        );
        if self
            .current_provider
            .as_ref()
            .is_some_and(|selected| selected != &provider)
        {
            self.errors.push(diagnostic(
                DiagnosticCode::TYPE_MISMATCH,
                "declaration descriptors on one class must use the same canonical provider"
                    .to_string(),
                call.range(),
            ));
            return;
        }
        self.current_provider = Some(provider);
        let target_callable = if expected_kind == DeclarationDescriptorKind::Method {
            let prefix = format!("{owner}.");
            let method = target_identity
                .strip_prefix(&prefix)
                .and_then(|identity| identity.split_once(':').map(|(method, _)| method))
                .unwrap_or(target_identity.as_str());
            if let Some(identity) = crate::callable_identities::method_declaration(
                self.module_name,
                self.result,
                owner,
                method,
            ) {
                Some(identity)
            } else {
                self.errors.push(diagnostic(
                    DiagnosticCode::META_MALFORMED_DECLARATION,
                    format!(
                        "method descriptor '{}' target does not have a checked method identity",
                        declaration.function
                    ),
                    call.range(),
                ));
                return;
            }
        } else {
            None
        };
        match self.evaluate_call(declaration, call) {
            Ok(value) => self.descriptors.push(TypedDeclarationDescriptor {
                owner: owner.to_string(),
                target_kind: expected_kind,
                target_identity,
                target_callable,
                provider_module: declaration.provider_module.clone(),
                provider_function: declaration.provider_function.clone(),
                value_type: declaration.return_type.clone(),
                value,
                range: call.range(),
            }),
            Err(error) => self.errors.push(*error),
        }
    }

    fn evaluate_call(
        &self,
        declaration: &DeclarationDescriptorFunction,
        call: &ExprCall,
    ) -> DescriptorResult<StaticProgramValue> {
        let function_type = self.function_type(declaration).ok_or_else(|| {
            boxed_malformed("descriptor function signature is unavailable", call.range())
        })?;
        let arguments = self.arguments(declaration, &function_type, call)?;
        let functions = self.const_functions(declaration).ok_or_else(|| {
            boxed_malformed(
                "descriptor package exports no const functions",
                call.range(),
            )
        })?;
        let evaluated = DeterministicConstEvaluator::new(&functions)
            .evaluate_function(&declaration.function, arguments)
            .map_err(|error| {
                Box::new(diagnostic(
                    DiagnosticCode::META_MALFORMED_DECLARATION,
                    format!(
                        "typed descriptor '{}' failed bounded const evaluation: {}",
                        declaration.function, error.detail
                    ),
                    call.range(),
                ))
            })?;
        if !const_value_assignable(&evaluated, &declaration.return_type) {
            return Err(boxed_malformed(
                "descriptor const result does not match its checked return type",
                call.range(),
            ));
        }
        crate::specialization_support::static_program_value(&evaluated)
            .map_err(|problem| boxed_malformed(problem, call.range()))
    }

    fn function_type(&self, declaration: &DeclarationDescriptorFunction) -> Option<FunctionType> {
        if declaration.module == self.module_name {
            let function = self
                .result
                .module
                .functions
                .iter()
                .find(|function| function.name == declaration.function)?;
            return Some(function_type(function));
        }
        self.external_defs
            .functions
            .get(&declaration.module)?
            .get(&declaration.function)
            .cloned()
    }

    fn arguments(
        &self,
        declaration: &DeclarationDescriptorFunction,
        function: &FunctionType,
        call: &ExprCall,
    ) -> DescriptorResult<Vec<ConstValue>> {
        if call.arguments.args.len() > function.params.len() {
            return Err(boxed_malformed(
                "descriptor call has too many positional arguments",
                call.range(),
            ));
        }
        let mut values = vec![None; function.params.len()];
        for (index, expression) in call.arguments.args.iter().enumerate() {
            values[index] = Some(self.argument_value(expression, &function.params[index].1)?);
        }
        for keyword in &call.arguments.keywords {
            let Some(name) = &keyword.arg else {
                return Err(boxed_malformed(
                    "descriptor calls do not accept dictionary expansion",
                    keyword.range(),
                ));
            };
            let Some(index) = function
                .params
                .iter()
                .position(|(parameter, _, _)| parameter == name.as_str())
            else {
                return Err(boxed_malformed(
                    format!("descriptor call has no parameter named '{name}'"),
                    keyword.range(),
                ));
            };
            if values[index].is_some() {
                return Err(boxed_malformed(
                    format!("descriptor parameter '{name}' is supplied more than once"),
                    keyword.range(),
                ));
            }
            values[index] = Some(self.argument_value(&keyword.value, &function.params[index].1)?);
        }
        let defaults = self.function_defaults(declaration);
        for (index, value) in values.iter_mut().enumerate() {
            if value.is_none() {
                *value = defaults
                    .iter()
                    .find(|(default_index, _)| *default_index == index)
                    .and_then(|(_, expression)| {
                        crate::structural_shape::const_value_from_hir(expression)
                    });
            }
        }
        values
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                value.ok_or_else(|| {
                    boxed_malformed(
                        format!(
                            "descriptor call is missing required parameter '{}'",
                            function.params[index].0
                        ),
                        call.range(),
                    )
                })
            })
            .collect()
    }

    fn argument_value(&self, expression: &Expr, expected: &Type) -> DescriptorResult<ConstValue> {
        if let Some(value) = literal_const_value(expression) {
            if const_value_assignable(&value, expected) || static_value_expected(expected) {
                return Ok(value);
            }
        }
        if callable_expected(expected) {
            let identity = crate::callable_identities::resolve(
                self.module_name,
                self.result,
                self.external_defs,
                expression,
            )
            .ok_or_else(|| {
                boxed_malformed(
                    "descriptor callable argument is not a checked callable",
                    expression.range(),
                )
            })?;
            return Ok(ConstValue::CallableIdentity(identity));
        }
        if let Some(item) = list_item_type(expected) {
            if let Expr::List(list) = expression {
                return list
                    .elts
                    .iter()
                    .map(|element| self.argument_value(element, item))
                    .collect::<DescriptorResult<Vec<_>>>()
                    .map(ConstValue::List);
            }
        }
        if let Expr::Call(call) = expression {
            return self.nested_const_call(call, expected);
        }
        let value = literal_const_value(expression).ok_or_else(|| {
            boxed_malformed(
                "descriptor argument is not a bounded const value",
                expression.range(),
            )
        })?;
        if const_value_assignable(&value, expected) {
            Ok(value)
        } else {
            Err(Box::new(diagnostic(
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "descriptor argument is not assignable to parameter type '{}'",
                    expected.display_name()
                ),
                expression.range(),
            )))
        }
    }

    fn const_call_target(&self, local_name: &str) -> Option<(String, HirFunction, HirModule)> {
        if let Some(function) = self.result.module.functions.iter().find(|function| {
            function.name == local_name
                && function.decorators.iter().any(|item| item == "const_eval")
        }) {
            let mut functions = self
                .result
                .module
                .functions
                .iter()
                .filter(|function| function.decorators.iter().any(|item| item == "const_eval"))
                .cloned()
                .collect::<Vec<_>>();
            functions.sort_by(|left, right| left.name.cmp(&right.name));
            return Some((
                self.module_name.to_string(),
                function.clone(),
                HirModule {
                    functions,
                    classes: Vec::new(),
                    imports: Vec::new(),
                    constants: Vec::new(),
                    generic_functions: HashMap::new(),
                    type_param_bounds: HashMap::new(),
                },
            ));
        }
        for import in &self.result.module.imports {
            for original in &import.names {
                let local = import
                    .aliases
                    .iter()
                    .find(|(name, _)| name == original)
                    .map_or(original.as_str(), |(_, alias)| alias.as_str());
                if local != local_name {
                    continue;
                }
                let exported = self.external_defs.const_functions.get(&import.module)?;
                let function = exported.get(original)?.clone();
                let mut names = exported.keys().collect::<Vec<_>>();
                names.sort();
                let functions = names
                    .into_iter()
                    .filter_map(|name| exported.get(name).cloned())
                    .collect();
                return Some((
                    import.module.clone(),
                    function,
                    HirModule {
                        functions,
                        classes: Vec::new(),
                        imports: Vec::new(),
                        constants: Vec::new(),
                        generic_functions: HashMap::new(),
                        type_param_bounds: HashMap::new(),
                    },
                ));
            }
        }
        None
    }

    fn function_defaults(
        &self,
        declaration: &DeclarationDescriptorFunction,
    ) -> Vec<(usize, sifr_lowering::HirExpr)> {
        if declaration.module == self.module_name {
            return self
                .result
                .function_defaults
                .get(&declaration.function)
                .cloned()
                .unwrap_or_default();
        }
        self.external_defs
            .function_defaults
            .get(&declaration.module)
            .and_then(|defaults| defaults.get(&declaration.function))
            .cloned()
            .unwrap_or_default()
    }

    fn const_functions(&self, declaration: &DeclarationDescriptorFunction) -> Option<HirModule> {
        let mut functions = if declaration.module == self.module_name {
            self.result
                .module
                .functions
                .iter()
                .filter(|function| function.decorators.iter().any(|item| item == "const_eval"))
                .cloned()
                .collect::<Vec<_>>()
        } else {
            let functions = self
                .external_defs
                .const_functions
                .get(&declaration.module)?;
            let mut names = functions.keys().collect::<Vec<_>>();
            names.sort();
            names
                .into_iter()
                .filter_map(|name| functions.get(name).cloned())
                .collect()
        };
        functions.sort_by(|left, right| left.name.cmp(&right.name));
        Some(HirModule {
            functions,
            classes: Vec::new(),
            imports: Vec::new(),
            constants: Vec::new(),
            generic_functions: HashMap::new(),
            type_param_bounds: HashMap::new(),
        })
    }
}

struct InvalidLocationVisitor<'a> {
    resolver: &'a DescriptorResolver,
    accepted: &'a HashSet<TextRange>,
    errors: Vec<HirDiagnostic>,
}

impl<'a> Visitor<'a> for InvalidLocationVisitor<'_> {
    fn visit_expr(&mut self, expression: &'a Expr) {
        if let Some((declaration, call)) = self.resolver.call(expression) {
            if !self.accepted.contains(&call.range()) {
                self.errors.push(malformed(
                    format!(
                        "{} descriptor '{}' is not valid in this declaration location",
                        descriptor_kind_name(declaration.kind),
                        declaration.function
                    ),
                    call.range(),
                ));
            }
        }
        visitor::walk_expr(self, expression);
    }
}

fn function_type(function: &HirFunction) -> FunctionType {
    FunctionType {
        receiver: function.receiver,
        params: function
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
        return_type: Box::new(function.return_type.clone()),
    }
}

fn literal_const_value(expression: &Expr) -> Option<ConstValue> {
    match expression {
        Expr::NoneLiteral(_) => Some(ConstValue::None),
        Expr::BooleanLiteral(value) => Some(ConstValue::Bool(value.value)),
        Expr::NumberLiteral(value) => match &value.value {
            Number::Int(value) => value.as_i64().map(BigInt::from).map(ConstValue::Integer),
            Number::Float(value) => Some(ConstValue::FloatBits(value.to_bits())),
            Number::Complex { .. } => None,
        },
        Expr::StringLiteral(value) => Some(ConstValue::String(value.value.to_str().to_string())),
        Expr::UnaryOp(unary) if matches!(unary.op, sifr_python_ast::UnaryOp::USub) => {
            let ConstValue::Integer(value) = literal_const_value(&unary.operand)? else {
                return None;
            };
            Some(ConstValue::Integer(-value))
        }
        Expr::List(list) => list
            .elts
            .iter()
            .map(literal_const_value)
            .collect::<Option<Vec<_>>>()
            .map(ConstValue::List),
        Expr::Tuple(tuple) => tuple
            .elts
            .iter()
            .map(literal_const_value)
            .collect::<Option<Vec<_>>>()
            .map(ConstValue::Tuple),
        Expr::Dict(dict) => {
            let mut values = BTreeMap::new();
            for item in &dict.items {
                let key = item.key.as_ref()?;
                let Expr::StringLiteral(key) = key else {
                    return None;
                };
                values.insert(
                    key.value.to_str().to_string(),
                    literal_const_value(&item.value)?,
                );
            }
            Some(ConstValue::Record(values))
        }
        _ => None,
    }
}

pub(crate) fn const_value_assignable(value: &ConstValue, expected: &Type) -> bool {
    match expected.resolve_alias() {
        Type::Union(members) => members
            .iter()
            .any(|member| const_value_assignable(value, member)),
        Type::None => matches!(value, ConstValue::None),
        Type::Bool => matches!(value, ConstValue::Bool(_)),
        Type::Int | Type::FixedInt(_) => matches!(value, ConstValue::Integer(_)),
        Type::Float => matches!(value, ConstValue::FloatBits(_) | ConstValue::Integer(_)),
        Type::Str => matches!(value, ConstValue::String(_)),
        Type::Bytes => matches!(value, ConstValue::Bytes(_)),
        Type::List(item) => {
            matches!(value, ConstValue::List(values) if values.iter().all(|value| const_value_assignable(value, item)))
        }
        Type::Tuple(items) => {
            matches!(value, ConstValue::Tuple(values) if values.len() == items.len() && values.iter().zip(items).all(|(value, item)| const_value_assignable(value, item)))
        }
        Type::Dict(key, item) => {
            matches!(value, ConstValue::Record(values) if matches!(key.resolve_alias(), Type::Str) && values.values().all(|value| const_value_assignable(value, item)))
        }
        Type::Class { identity, name, .. }
            if name == "CallableIdentity"
                && identity.as_deref() == Some("sifr.meta.CallableIdentity") =>
        {
            matches!(value, ConstValue::CallableIdentity(_))
        }
        Type::Class { identity, name, .. }
            if name == "StaticValue" && identity.as_deref() == Some("sifr.meta.StaticValue") =>
        {
            !matches!(value, ConstValue::SourceOrigin(_))
        }
        Type::Class { fields, .. } => matches!(value, ConstValue::Record(values)
            if values.len() == fields.len()
                && fields.iter().all(|(name, field)| values
                    .get(name)
                    .is_some_and(|value| const_value_assignable(value, field)))),
        Type::LiteralInt(expected) => {
            matches!(value, ConstValue::Integer(value) if value == &BigInt::from(*expected))
        }
        Type::LiteralStr(expected) => {
            matches!(value, ConstValue::String(value) if value == expected)
        }
        Type::LiteralBool(expected) => {
            matches!(value, ConstValue::Bool(value) if value == expected)
        }
        _ => false,
    }
}

fn callable_expected(expected: &Type) -> bool {
    match expected.resolve_alias() {
        Type::Callable(_, _, _) | Type::Function(_) => true,
        Type::Class { identity, name, .. } => {
            name == "CallableIdentity" && identity.as_deref() == Some("sifr.meta.CallableIdentity")
        }
        Type::Union(members) => members.iter().any(callable_expected),
        _ => false,
    }
}

fn list_item_type(expected: &Type) -> Option<&Type> {
    match expected.resolve_alias() {
        Type::List(item) => Some(item),
        Type::Union(members) => members.iter().find_map(list_item_type),
        _ => None,
    }
}

fn static_value_expected(expected: &Type) -> bool {
    match expected.resolve_alias() {
        Type::Class { identity, name, .. } => {
            name == "StaticValue" && identity.as_deref() == Some("sifr.meta.StaticValue")
        }
        Type::Union(members) => members.iter().any(static_value_expected),
        _ => false,
    }
}

fn descriptor_kind_name(kind: DeclarationDescriptorKind) -> &'static str {
    match kind {
        DeclarationDescriptorKind::Field => "field",
        DeclarationDescriptorKind::Class => "class",
        DeclarationDescriptorKind::Method => "method",
        DeclarationDescriptorKind::Type => "type",
    }
}

fn malformed(problem: impl Into<String>, range: TextRange) -> HirDiagnostic {
    diagnostic(
        DiagnosticCode::META_MALFORMED_DECLARATION,
        format!("malformed typed declaration descriptor: {}", problem.into()),
        range,
    )
}

fn boxed_malformed(problem: impl Into<String>, range: TextRange) -> Box<HirDiagnostic> {
    Box::new(malformed(problem, range))
}

fn diagnostic(code: DiagnosticCode, message: String, range: TextRange) -> HirDiagnostic {
    HirDiagnostic {
        code: Some(code),
        message,
        args: BTreeMap::new(),
        help: None,
        primary_range: Some(range),
        line: None,
        col: None,
    }
}
