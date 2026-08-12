//! Bounded deterministic evaluation of explicitly const Sifr functions.

use crate::ConstValue;
use num_bigint::BigInt;
use sifr_lowering::{HirExpr, HirFunction, HirIteratorOp, HirModule, HirStmt};
use sifr_type_system::Type;
use std::collections::BTreeMap;

const DEFAULT_STEP_LIMIT: usize = 100_000;
const DEFAULT_RECURSION_LIMIT: usize = 64;
const DEFAULT_COLLECTION_LIMIT: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstEvalErrorKind {
    FunctionNotFound,
    FunctionNotConst,
    ArgumentCount,
    UnsupportedExpression,
    UnsupportedStatement,
    UnknownBinding,
    TypeMismatch,
    DivisionByZero,
    IndexOutOfBounds,
    StepLimit,
    RecursionLimit,
    CollectionLimit,
    ExplicitFailure,
    MissingReturn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstEvalError {
    pub kind: ConstEvalErrorKind,
    pub detail: String,
}

pub struct DeterministicConstEvaluator<'a> {
    module: &'a HirModule,
    remaining_steps: usize,
    recursion_limit: usize,
    collection_limit: usize,
}

enum Control {
    Next,
    Return(ConstValue),
    Break,
    Continue,
}

type Environment = BTreeMap<String, ConstValue>;

impl<'a> DeterministicConstEvaluator<'a> {
    #[must_use]
    pub fn new(module: &'a HirModule) -> Self {
        Self {
            module,
            remaining_steps: DEFAULT_STEP_LIMIT,
            recursion_limit: DEFAULT_RECURSION_LIMIT,
            collection_limit: DEFAULT_COLLECTION_LIMIT,
        }
    }

    #[must_use]
    pub fn with_limits(
        module: &'a HirModule,
        step_limit: usize,
        recursion_limit: usize,
        collection_limit: usize,
    ) -> Self {
        Self {
            module,
            remaining_steps: step_limit,
            recursion_limit,
            collection_limit,
        }
    }

    pub fn evaluate_function(
        &mut self,
        name: &str,
        arguments: Vec<ConstValue>,
    ) -> Result<ConstValue, ConstEvalError> {
        self.call(name, arguments, 0)
    }

    fn call(
        &mut self,
        name: &str,
        arguments: Vec<ConstValue>,
        depth: usize,
    ) -> Result<ConstValue, ConstEvalError> {
        if depth > self.recursion_limit {
            return error(
                ConstEvalErrorKind::RecursionLimit,
                "const recursion limit exceeded",
            );
        }
        let function = self
            .module
            .functions
            .iter()
            .find(|function| function.name == name)
            .cloned()
            .ok_or_else(|| ConstEvalError {
                kind: ConstEvalErrorKind::FunctionNotFound,
                detail: format!("const function '{name}' was not found"),
            })?;
        if !function.decorators.iter().any(|item| item == "const_eval") {
            return error(
                ConstEvalErrorKind::FunctionNotConst,
                format!("function '{name}' is not declared @const_eval"),
            );
        }
        if function.params.len() != arguments.len() {
            return error(
                ConstEvalErrorKind::ArgumentCount,
                format!(
                    "function '{name}' expects {} arguments, received {}",
                    function.params.len(),
                    arguments.len()
                ),
            );
        }
        let mut environment = function
            .params
            .iter()
            .zip(arguments)
            .map(|(parameter, value)| (parameter.name.clone(), value))
            .collect();
        match self.eval_block(&function, &function.body, &mut environment, depth)? {
            Control::Return(value) => Ok(value),
            Control::Next => error(
                ConstEvalErrorKind::MissingReturn,
                format!("const function '{name}' completed without a value"),
            ),
            Control::Break | Control::Continue => error(
                ConstEvalErrorKind::UnsupportedStatement,
                "loop control escaped a const function",
            ),
        }
    }

    fn eval_block(
        &mut self,
        function: &HirFunction,
        statements: &[HirStmt],
        environment: &mut Environment,
        depth: usize,
    ) -> Result<Control, ConstEvalError> {
        for statement in statements {
            self.step()?;
            let control = self.eval_statement(function, statement, environment, depth)?;
            if !matches!(control, Control::Next) {
                return Ok(control);
            }
        }
        Ok(Control::Next)
    }

    fn eval_statement(
        &mut self,
        function: &HirFunction,
        statement: &HirStmt,
        environment: &mut Environment,
        depth: usize,
    ) -> Result<Control, ConstEvalError> {
        match statement {
            HirStmt::Let { name, value, .. } | HirStmt::Assign { name, value } => {
                let value = self.eval_expr(value, environment, depth)?;
                environment.insert(name.clone(), value);
                Ok(Control::Next)
            }
            HirStmt::AugAssign { name, op, value } => {
                let left = environment
                    .get(name)
                    .cloned()
                    .ok_or_else(|| unknown_binding(name))?;
                let right = self.eval_expr(value, environment, depth)?;
                let binary_op = op.strip_suffix('=').unwrap_or(op);
                let value = binary(binary_op, left, right)?;
                environment.insert(name.clone(), value);
                Ok(Control::Next)
            }
            HirStmt::Return { value } => Ok(Control::Return(match value {
                Some(value) => self.eval_expr(value, environment, depth)?,
                None => ConstValue::None,
            })),
            HirStmt::Expr { expr } => {
                let _ = self.eval_expr(expr, environment, depth)?;
                Ok(Control::Next)
            }
            HirStmt::If {
                condition,
                then_body,
                elif_clauses,
                else_body,
            } => {
                if truthy(&self.eval_expr(condition, environment, depth)?) {
                    return self.eval_block(function, then_body, environment, depth);
                }
                for (condition, body) in elif_clauses {
                    if truthy(&self.eval_expr(condition, environment, depth)?) {
                        return self.eval_block(function, body, environment, depth);
                    }
                }
                if let Some(body) = else_body {
                    return self.eval_block(function, body, environment, depth);
                }
                Ok(Control::Next)
            }
            HirStmt::While {
                condition,
                body,
                else_body,
            } => loop {
                self.step()?;
                if !truthy(&self.eval_expr(condition, environment, depth)?) {
                    if let Some(body) = else_body {
                        return self.eval_block(function, body, environment, depth);
                    }
                    return Ok(Control::Next);
                }
                match self.eval_block(function, body, environment, depth)? {
                    Control::Next | Control::Continue => {}
                    Control::Break => return Ok(Control::Next),
                    returned @ Control::Return(_) => return Ok(returned),
                }
            },
            HirStmt::For {
                target,
                iter,
                body,
                else_body,
                ..
            } => {
                let values = sequence(self.eval_expr(iter, environment, depth)?)?;
                self.check_collection(values.len())?;
                for value in values {
                    environment.insert(target.clone(), value);
                    match self.eval_block(function, body, environment, depth)? {
                        Control::Next | Control::Continue => {}
                        Control::Break => return Ok(Control::Next),
                        returned @ Control::Return(_) => return Ok(returned),
                    }
                }
                if let Some(body) = else_body {
                    return self.eval_block(function, body, environment, depth);
                }
                Ok(Control::Next)
            }
            HirStmt::Break => Ok(Control::Break),
            HirStmt::Continue => Ok(Control::Continue),
            HirStmt::Pass => Ok(Control::Next),
            HirStmt::Assert { test, .. } => {
                if truthy(&self.eval_expr(test, environment, depth)?) {
                    Ok(Control::Next)
                } else {
                    error(
                        ConstEvalErrorKind::ExplicitFailure,
                        "const assertion failed",
                    )
                }
            }
            HirStmt::Raise { .. } => error(
                ConstEvalErrorKind::ExplicitFailure,
                "const function raised an error",
            ),
            _ => error(
                ConstEvalErrorKind::UnsupportedStatement,
                "statement is not supported by deterministic const evaluation",
            ),
        }
    }

    fn eval_expr(
        &mut self,
        expression: &HirExpr,
        environment: &mut Environment,
        depth: usize,
    ) -> Result<ConstValue, ConstEvalError> {
        self.step()?;
        match expression {
            HirExpr::IntLiteral(value) => Ok(ConstValue::Integer(BigInt::from(*value))),
            HirExpr::LargeIntLiteral(value) => value
                .parse()
                .map(ConstValue::Integer)
                .map_err(|_| type_mismatch("invalid exact integer literal")),
            HirExpr::FloatLiteral(value) => Ok(ConstValue::FloatBits(value.to_bits())),
            HirExpr::StringLiteral(value) => Ok(ConstValue::String(value.clone())),
            HirExpr::BoolLiteral(value) => Ok(ConstValue::Bool(*value)),
            HirExpr::NoneLiteral => Ok(ConstValue::None),
            HirExpr::Name { name, .. } => environment
                .get(name)
                .cloned()
                .ok_or_else(|| unknown_binding(name)),
            HirExpr::UnaryOp { op, operand, .. } => {
                unary(op, self.eval_expr(operand, environment, depth)?)
            }
            HirExpr::BinOp {
                left, op, right, ..
            } => {
                let left = self.eval_expr(left, environment, depth)?;
                let right = self.eval_expr(right, environment, depth)?;
                binary(op, left, right)
            }
            HirExpr::Compare {
                left,
                ops,
                comparators,
                ..
            } => {
                let mut left = self.eval_expr(left, environment, depth)?;
                for (op, right) in ops.iter().zip(comparators) {
                    let right = self.eval_expr(right, environment, depth)?;
                    if !compare(op, &left, &right)? {
                        return Ok(ConstValue::Bool(false));
                    }
                    left = right;
                }
                Ok(ConstValue::Bool(true))
            }
            HirExpr::BoolOp { op, values, .. } => {
                let is_and = op == "and";
                for value in values {
                    let result = truthy(&self.eval_expr(value, environment, depth)?);
                    if (is_and && !result) || (!is_and && result) {
                        return Ok(ConstValue::Bool(!is_and));
                    }
                }
                Ok(ConstValue::Bool(is_and))
            }
            HirExpr::Call { func, args, .. } => {
                let args = args
                    .iter()
                    .map(|argument| self.eval_expr(argument, environment, depth))
                    .collect::<Result<Vec<_>, _>>()?;
                match func.as_str() {
                    "len" => len(&args),
                    "str" => stringify(&args),
                    _ => self.call(func, args, depth + 1),
                }
            }
            HirExpr::IteratorCall {
                op: HirIteratorOp::Iter,
                args,
                ..
            } if args.len() == 1 => self.eval_expr(&args[0], environment, depth),
            HirExpr::IfExpr {
                condition,
                then_expr,
                else_expr,
                ..
            } => {
                if truthy(&self.eval_expr(condition, environment, depth)?) {
                    self.eval_expr(then_expr, environment, depth)
                } else {
                    self.eval_expr(else_expr, environment, depth)
                }
            }
            HirExpr::ListLiteral { elements, ty } if matches!(ty.resolve_alias(), Type::Bytes) => {
                let values = self.eval_elements(elements, environment, depth)?;
                values
                    .into_iter()
                    .map(|value| match value {
                        ConstValue::Integer(value) => value
                            .to_string()
                            .parse::<u8>()
                            .map_err(|_| type_mismatch("byte literal element is out of range")),
                        _ => Err(type_mismatch("byte literal element is not an integer")),
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map(ConstValue::Bytes)
            }
            HirExpr::ListLiteral { elements, .. } => self
                .eval_elements(elements, environment, depth)
                .map(ConstValue::List),
            HirExpr::TupleLiteral { elements, .. } => self
                .eval_elements(elements, environment, depth)
                .map(ConstValue::Tuple),
            HirExpr::DictLiteral { keys, values, .. } => {
                self.check_collection(keys.len())?;
                let mut result = BTreeMap::new();
                for (key, value) in keys.iter().zip(values) {
                    let ConstValue::String(key) = self.eval_expr(key, environment, depth)? else {
                        return error(
                            ConstEvalErrorKind::TypeMismatch,
                            "const record keys must be strings",
                        );
                    };
                    result.insert(key, self.eval_expr(value, environment, depth)?);
                }
                Ok(ConstValue::Record(result))
            }
            HirExpr::Index { object, index, .. } => {
                let object = self.eval_expr(object, environment, depth)?;
                let index = self.eval_expr(index, environment, depth)?;
                index_value(object, index)
            }
            HirExpr::FieldAccess { object, field, .. } => {
                let ConstValue::Record(values) = self.eval_expr(object, environment, depth)? else {
                    return error(
                        ConstEvalErrorKind::TypeMismatch,
                        "const field access requires a record",
                    );
                };
                values.get(field).cloned().ok_or_else(|| ConstEvalError {
                    kind: ConstEvalErrorKind::UnknownBinding,
                    detail: format!("const record has no field '{field}'"),
                })
            }
            HirExpr::ConstructorCall { args, ty, .. } => {
                let Type::Class { fields, .. } = ty.resolve_alias() else {
                    return error(
                        ConstEvalErrorKind::UnsupportedExpression,
                        "const construction requires a structural class",
                    );
                };
                if fields.len() != args.len() {
                    return error(
                        ConstEvalErrorKind::ArgumentCount,
                        "const constructor argument count does not match its fields",
                    );
                }
                let mut record = BTreeMap::new();
                for ((name, _), argument) in fields.iter().zip(args) {
                    record.insert(name.clone(), self.eval_expr(argument, environment, depth)?);
                }
                Ok(ConstValue::Record(record))
            }
            HirExpr::MethodCall {
                object,
                method,
                args,
                ..
            } if method == "len" && args.is_empty() => {
                let value = self.eval_expr(object, environment, depth)?;
                len(&[value])
            }
            HirExpr::MethodCall {
                object,
                method,
                args,
                ..
            } if method == "append" && args.len() == 1 => {
                let HirExpr::Name { name, .. } = object.as_ref() else {
                    return error(
                        ConstEvalErrorKind::UnsupportedExpression,
                        "const append receiver must be a local list",
                    );
                };
                let value = self.eval_expr(&args[0], environment, depth)?;
                let Some(ConstValue::List(values)) = environment.get_mut(name) else {
                    return error(
                        ConstEvalErrorKind::TypeMismatch,
                        "const append receiver must be a list",
                    );
                };
                if values.len() >= self.collection_limit {
                    return error(
                        ConstEvalErrorKind::CollectionLimit,
                        "const collection limit exceeded",
                    );
                }
                values.push(value);
                Ok(ConstValue::None)
            }
            _ => error(
                ConstEvalErrorKind::UnsupportedExpression,
                "expression is not supported by deterministic const evaluation",
            ),
        }
    }

    fn eval_elements(
        &mut self,
        elements: &[HirExpr],
        environment: &mut Environment,
        depth: usize,
    ) -> Result<Vec<ConstValue>, ConstEvalError> {
        self.check_collection(elements.len())?;
        elements
            .iter()
            .map(|element| self.eval_expr(element, environment, depth))
            .collect()
    }

    fn step(&mut self) -> Result<(), ConstEvalError> {
        if self.remaining_steps == 0 {
            return error(
                ConstEvalErrorKind::StepLimit,
                "const evaluation step limit exceeded",
            );
        }
        self.remaining_steps -= 1;
        Ok(())
    }

    fn check_collection(&self, length: usize) -> Result<(), ConstEvalError> {
        if length > self.collection_limit {
            error(
                ConstEvalErrorKind::CollectionLimit,
                "const collection limit exceeded",
            )
        } else {
            Ok(())
        }
    }
}

fn unary(op: &str, value: ConstValue) -> Result<ConstValue, ConstEvalError> {
    match (op, value) {
        ("-", ConstValue::Integer(value)) => Ok(ConstValue::Integer(-value)),
        ("+", ConstValue::Integer(value)) => Ok(ConstValue::Integer(value)),
        ("not", value) => Ok(ConstValue::Bool(!truthy(&value))),
        _ => error(
            ConstEvalErrorKind::TypeMismatch,
            "unsupported const unary operation",
        ),
    }
}

fn binary(op: &str, left: ConstValue, right: ConstValue) -> Result<ConstValue, ConstEvalError> {
    match (op, left, right) {
        ("+", ConstValue::Integer(left), ConstValue::Integer(right)) => {
            Ok(ConstValue::Integer(left + right))
        }
        ("-", ConstValue::Integer(left), ConstValue::Integer(right)) => {
            Ok(ConstValue::Integer(left - right))
        }
        ("*", ConstValue::Integer(left), ConstValue::Integer(right)) => {
            Ok(ConstValue::Integer(left * right))
        }
        ("//" | "%", ConstValue::Integer(_), ConstValue::Integer(ref right))
            if right == &BigInt::from(0) =>
        {
            error(
                ConstEvalErrorKind::DivisionByZero,
                "const integer division by zero",
            )
        }
        ("//", ConstValue::Integer(left), ConstValue::Integer(right)) => {
            let (quotient, _) = floor_div_mod(&left, right);
            Ok(ConstValue::Integer(quotient))
        }
        ("%", ConstValue::Integer(left), ConstValue::Integer(right)) => {
            let (_, remainder) = floor_div_mod(&left, right);
            Ok(ConstValue::Integer(remainder))
        }
        ("+", ConstValue::String(left), ConstValue::String(right)) => {
            Ok(ConstValue::String(left + &right))
        }
        ("+", ConstValue::List(mut left), ConstValue::List(right)) => {
            left.extend(right);
            Ok(ConstValue::List(left))
        }
        _ => error(
            ConstEvalErrorKind::TypeMismatch,
            "unsupported const binary operation",
        ),
    }
}

fn floor_div_mod(left: &BigInt, right: BigInt) -> (BigInt, BigInt) {
    let zero = BigInt::from(0);
    let mut quotient = left / &right;
    let mut remainder = left % &right;
    if remainder != zero && ((remainder < zero) != (right < zero)) {
        quotient -= 1;
        remainder += right;
    }
    (quotient, remainder)
}

fn compare(op: &str, left: &ConstValue, right: &ConstValue) -> Result<bool, ConstEvalError> {
    match op {
        "==" => Ok(left == right),
        "!=" => Ok(left != right),
        "is" => Ok(left == right),
        "is not" => Ok(left != right),
        "<" | "<=" | ">" | ">=" => match (left, right) {
            (ConstValue::Integer(left), ConstValue::Integer(right)) => Ok(match op {
                "<" => left < right,
                "<=" => left <= right,
                ">" => left > right,
                _ => left >= right,
            }),
            (ConstValue::String(left), ConstValue::String(right)) => Ok(match op {
                "<" => left < right,
                "<=" => left <= right,
                ">" => left > right,
                _ => left >= right,
            }),
            _ => error(
                ConstEvalErrorKind::TypeMismatch,
                "const values are not order-comparable",
            ),
        },
        _ => error(
            ConstEvalErrorKind::UnsupportedExpression,
            "unsupported const comparison",
        ),
    }
}

fn len(arguments: &[ConstValue]) -> Result<ConstValue, ConstEvalError> {
    if arguments.len() != 1 {
        return error(
            ConstEvalErrorKind::ArgumentCount,
            "len expects one const argument",
        );
    }
    let length = match &arguments[0] {
        ConstValue::String(value) => value.chars().count(),
        ConstValue::Bytes(value) => value.len(),
        ConstValue::Tuple(values) | ConstValue::List(values) => values.len(),
        ConstValue::Record(values) => values.len(),
        _ => {
            return error(
                ConstEvalErrorKind::TypeMismatch,
                "const value has no length",
            )
        }
    };
    Ok(ConstValue::Integer(BigInt::from(length)))
}

fn stringify(arguments: &[ConstValue]) -> Result<ConstValue, ConstEvalError> {
    if arguments.len() != 1 {
        return error(
            ConstEvalErrorKind::ArgumentCount,
            "str expects one const argument",
        );
    }
    let value = match &arguments[0] {
        ConstValue::None => "None".to_string(),
        ConstValue::Bool(value) => value.to_string(),
        ConstValue::Integer(value) => value.to_string(),
        ConstValue::String(value) => value.clone(),
        _ => {
            return error(
                ConstEvalErrorKind::TypeMismatch,
                "const value cannot be converted to str",
            )
        }
    };
    Ok(ConstValue::String(value))
}

fn index_value(object: ConstValue, index: ConstValue) -> Result<ConstValue, ConstEvalError> {
    match (object, index) {
        (ConstValue::Record(values), ConstValue::String(index)) => {
            values.get(&index).cloned().ok_or_else(|| ConstEvalError {
                kind: ConstEvalErrorKind::IndexOutOfBounds,
                detail: format!("const record has no key '{index}'"),
            })
        }
        (ConstValue::List(values) | ConstValue::Tuple(values), ConstValue::Integer(index)) => {
            let Ok(index) = index.to_string().parse::<usize>() else {
                return error(
                    ConstEvalErrorKind::IndexOutOfBounds,
                    "const index is out of bounds",
                );
            };
            values.get(index).cloned().ok_or_else(|| ConstEvalError {
                kind: ConstEvalErrorKind::IndexOutOfBounds,
                detail: format!("const index {index} is out of bounds"),
            })
        }
        _ => error(
            ConstEvalErrorKind::TypeMismatch,
            "const value is not indexable by this key",
        ),
    }
}

fn sequence(value: ConstValue) -> Result<Vec<ConstValue>, ConstEvalError> {
    match value {
        ConstValue::List(values) | ConstValue::Tuple(values) => Ok(values),
        _ => error(
            ConstEvalErrorKind::TypeMismatch,
            "const for loop requires a list or tuple",
        ),
    }
}

fn truthy(value: &ConstValue) -> bool {
    match value {
        ConstValue::None => false,
        ConstValue::Bool(value) => *value,
        ConstValue::Integer(value) => value != &BigInt::from(0),
        ConstValue::FloatBits(value) => f64::from_bits(*value) != 0.0,
        ConstValue::String(value) => !value.is_empty(),
        ConstValue::Bytes(value) => !value.is_empty(),
        ConstValue::Tuple(values) | ConstValue::List(values) => !values.is_empty(),
        ConstValue::Record(values) => !values.is_empty(),
    }
}

fn unknown_binding(name: &str) -> ConstEvalError {
    ConstEvalError {
        kind: ConstEvalErrorKind::UnknownBinding,
        detail: format!("unknown const binding '{name}'"),
    }
}

fn type_mismatch(detail: impl Into<String>) -> ConstEvalError {
    ConstEvalError {
        kind: ConstEvalErrorKind::TypeMismatch,
        detail: detail.into(),
    }
}

fn error<T>(kind: ConstEvalErrorKind, detail: impl Into<String>) -> Result<T, ConstEvalError> {
    Err(ConstEvalError {
        kind,
        detail: detail.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_lowering::lower_module;
    use sifr_syntax::parse_module_suite;

    fn lower(source: &str) -> sifr_lowering::LoweringResult {
        let parsed = parse_module_suite(source, None).expect("fixture parses");
        lower_module(&parsed).expect("fixture lowers")
    }

    #[test]
    fn evaluates_bounded_pure_const_function_deterministically() {
        let lowered = lower(
            "@const_eval\ndef describe(values: list[int]) -> int:\n    total: int = 0\n    for value in values:\n        total = total + value\n    return total\n",
        );
        let args = vec![ConstValue::List(vec![
            ConstValue::Integer(BigInt::from(2)),
            ConstValue::Integer(BigInt::from(5)),
        ])];
        let first = DeterministicConstEvaluator::new(&lowered.module)
            .evaluate_function("describe", args.clone())
            .expect("const evaluation succeeds");
        let second = DeterministicConstEvaluator::new(&lowered.module)
            .evaluate_function("describe", args)
            .expect("const evaluation succeeds");
        assert_eq!(first, ConstValue::Integer(BigInt::from(7)));
        assert_eq!(first, second);
    }

    #[test]
    fn fails_closed_on_unbounded_const_evaluation() {
        let lowered = lower(
            "@const_eval\ndef forever() -> int:\n    while True:\n        pass\n    return 0\n",
        );
        let error = DeterministicConstEvaluator::with_limits(&lowered.module, 20, 4, 8)
            .evaluate_function("forever", Vec::new())
            .expect_err("step budget is enforced");
        assert_eq!(error.kind, ConstEvalErrorKind::StepLimit);
    }

    #[test]
    fn augmented_assignment_preserves_floor_division_and_modulo_semantics() {
        let lowered = lower(
            "@const_eval\ndef arithmetic() -> tuple[int, int]:\n    quotient: int = -7\n    quotient += 0\n    quotient //= 3\n    remainder: int = -7\n    remainder %= 3\n    return (quotient, remainder)\n",
        );
        let value = DeterministicConstEvaluator::new(&lowered.module)
            .evaluate_function("arithmetic", Vec::new())
            .expect("const evaluation succeeds");
        assert_eq!(
            value,
            ConstValue::Tuple(vec![
                ConstValue::Integer(BigInt::from(-3)),
                ConstValue::Integer(BigInt::from(2)),
            ])
        );
    }

    #[test]
    fn preserves_bytes_as_the_closed_bytes_const_variant() {
        let lowered = lower("@const_eval\ndef payload() -> bytes:\n    return b\"typed\"\n");
        let value = DeterministicConstEvaluator::new(&lowered.module)
            .evaluate_function("payload", Vec::new())
            .expect("byte const evaluation succeeds");
        assert_eq!(value, ConstValue::Bytes(b"typed".to_vec()));
    }

    #[test]
    fn rejects_runtime_function_as_const_entrypoint() {
        let lowered = lower("def runtime() -> int:\n    return 1\n");
        let error = DeterministicConstEvaluator::new(&lowered.module)
            .evaluate_function("runtime", Vec::new())
            .expect_err("runtime function is rejected");
        assert_eq!(error.kind, ConstEvalErrorKind::FunctionNotConst);
    }
}
