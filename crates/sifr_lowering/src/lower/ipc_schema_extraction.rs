use super::ipc_payload_calls;
use sifr_ipc::{IpcSchemaField, IpcSchemaType, IpcSchemaVariant};
use sifr_type_system::Type;

pub(in crate::lower) fn extract_ipc_schema_type(ty: &Type) -> IpcSchemaType {
    extract_ipc_schema_type_inner(ty.resolve_alias())
}

fn extract_ipc_schema_type_inner(ty: &Type) -> IpcSchemaType {
    if ipc_payload_calls::non_ipc_serializable_reason(ty).is_some() {
        return unsupported_schema_type(ty);
    }
    match ty {
        Type::Bool | Type::LiteralBool(_) => IpcSchemaType::Bool,
        Type::Int | Type::FixedInt(_) | Type::LiteralInt(_) => IpcSchemaType::Int,
        Type::Float => IpcSchemaType::Float,
        Type::Str | Type::LiteralStr(_) => IpcSchemaType::Str,
        Type::Bytes => IpcSchemaType::Bytes,
        Type::None | Type::Never => IpcSchemaType::None,
        Type::List(inner) => IpcSchemaType::List(Box::new(extract_ipc_schema_type(inner))),
        Type::Dict(_, value) => IpcSchemaType::DictStr(Box::new(extract_ipc_schema_type(value))),
        Type::Result(ok, err) => IpcSchemaType::Result(
            Box::new(extract_ipc_schema_type(ok)),
            Box::new(extract_ipc_schema_type(err)),
        ),
        Type::Tuple(items) => IpcSchemaType::Tuple(
            items
                .iter()
                .map(extract_ipc_schema_type)
                .collect::<Vec<_>>(),
        ),
        Type::Union(_) => {
            extract_option_schema_type(ty).unwrap_or_else(|| IpcSchemaType::Unsupported {
                type_name: ty.display_name(),
            })
        }
        Type::Alias { body, .. } | Type::Newtype { inner: body, .. } => {
            extract_ipc_schema_type(body)
        }
        Type::Class { name, fields, .. } => IpcSchemaType::Record {
            name: name.clone(),
            fields: fields
                .iter()
                .map(|(field_name, field_ty)| IpcSchemaField {
                    name: field_name.clone(),
                    ty: extract_ipc_schema_type(field_ty),
                })
                .collect::<Vec<_>>(),
        },
        Type::StructuralRecord(record) => IpcSchemaType::Record {
            name: ty.display_name(),
            fields: record
                .fields()
                .iter()
                .map(|field| IpcSchemaField {
                    name: field.name().to_string(),
                    ty: extract_ipc_schema_type(field.ty()),
                })
                .collect(),
        },
        Type::Enum { name, variants, .. } => IpcSchemaType::Enum {
            name: name.clone(),
            variants: variants
                .iter()
                .map(|(variant_name, _)| IpcSchemaVariant {
                    name: variant_name.clone(),
                    payload: None,
                })
                .collect::<Vec<_>>(),
        },
        Type::Set(_)
        | Type::PythonBuffer(_)
        | Type::PythonArrow(_)
        | Type::PythonDlpackTensor(_)
        | Type::PythonDlpackStream
        | Type::Decimal
        | Type::BigDecimal
        | Type::Task(_, _)
        | Type::TaskResult(_, _)
        | Type::BlockingTask(_, _)
        | Type::JoinSet(_, _)
        | Type::Coroutine(_, _)
        | Type::Awaitable(_)
        | Type::AsyncIterator(_, _)
        | Type::AsyncGenerator(_, _)
        | Type::Failure(_)
        | Type::TimeoutResult(_)
        | Type::Select2(_, _)
        | Type::Function(_)
        | Type::AsyncFunction(_)
        | Type::Callable(..)
        | Type::AsyncCallable(..)
        | Type::Iterator(_)
        | Type::Iterable(_)
        | Type::Range
        | Type::Any
        | Type::Unknown
        | Type::TypeVar(_)
        | Type::Protocol { .. }
        | Type::Intersection(_)
        | Type::Template(_) => unsupported_schema_type(ty),
    }
}

fn extract_option_schema_type(ty: &Type) -> Option<IpcSchemaType> {
    let payload = ty.optional_member_type()?;
    Some(IpcSchemaType::Option(Box::new(extract_ipc_schema_type(
        &payload,
    ))))
}

fn unsupported_schema_type(ty: &Type) -> IpcSchemaType {
    IpcSchemaType::Unsupported {
        type_name: ty.display_name(),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/lower/ipc_schema_extraction/tests.rs"]
mod tests;
