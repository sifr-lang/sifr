use crate::Type;

pub(super) fn validate_codegen_type(ty: &Type) -> Result<(), crate::CodegenError> {
    match ty {
        Type::Callable(params, conventions, ret)
        | Type::AsyncCallable(params, conventions, ret) => {
            if params.len() != conventions.len() {
                return Err(crate::CodegenError::new(format!(
                    "unsupported callable type: {} parameters but {} conventions",
                    params.len(),
                    conventions.len()
                )));
            }
            for param in params {
                validate_codegen_type(param)?;
            }
            validate_codegen_type(ret)
        }
        Type::Function(function) | Type::AsyncFunction(function) => {
            for (_, param, _) in &function.params {
                validate_codegen_type(param)?;
            }
            validate_codegen_type(&function.return_type)
        }
        Type::List(inner)
        | Type::Set(inner)
        | Type::Iterable(inner)
        | Type::Iterator(inner)
        | Type::Awaitable(inner)
        | Type::Failure(inner)
        | Type::TimeoutResult(inner)
        | Type::PythonBuffer(inner)
        | Type::PythonDlpackTensor(inner)
        | Type::Newtype { inner, .. }
        | Type::Alias { body: inner, .. } => validate_codegen_type(inner),
        Type::Dict(left, right)
        | Type::Result(left, right)
        | Type::Coroutine(left, right)
        | Type::Task(left, right)
        | Type::TaskResult(left, right)
        | Type::Select2(left, right)
        | Type::BlockingTask(left, right)
        | Type::JoinSet(left, right)
        | Type::AsyncIterator(left, right)
        | Type::AsyncGenerator(left, right) => {
            validate_codegen_type(left)?;
            validate_codegen_type(right)
        }
        Type::Tuple(items) | Type::Union(items) | Type::Intersection(items) => {
            for item in items {
                validate_codegen_type(item)?;
            }
            Ok(())
        }
        Type::Class {
            type_args,
            fields,
            methods,
            ..
        } => {
            for type_arg in type_args {
                validate_codegen_type(type_arg)?;
            }
            for (_, field) in fields {
                validate_codegen_type(field)?;
            }
            for (_, method) in methods {
                validate_codegen_type(&Type::Function(method.clone()))?;
            }
            Ok(())
        }
        Type::Protocol { methods, .. } => {
            for (_, method) in methods {
                validate_codegen_type(&Type::Function(method.clone()))?;
            }
            Ok(())
        }
        Type::Int
        | Type::FixedInt(_)
        | Type::Float
        | Type::Bool
        | Type::Str
        | Type::Bytes
        | Type::None
        | Type::PythonArrow(_)
        | Type::PythonDlpackStream
        | Type::Range
        | Type::Any
        | Type::Never
        | Type::LiteralInt(_)
        | Type::LiteralStr(_)
        | Type::LiteralBool(_)
        | Type::Unknown
        | Type::TypeVar(_)
        | Type::Enum { .. }
        | Type::Decimal
        | Type::BigDecimal => Ok(()),
    }
}
