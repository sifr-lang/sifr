use super::ipc_payload_calls;
use sifr_stdlib::{IpcSchemaField, IpcSchemaType, IpcSchemaVariant};
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
        Type::Union(members) => {
            extract_option_schema_type(members).unwrap_or_else(|| IpcSchemaType::Unsupported {
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
        Type::Enum { name, variants } => IpcSchemaType::Enum {
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
        | Type::BigInt
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
        | Type::Iterator(_)
        | Type::Iterable(_)
        | Type::Range
        | Type::Any
        | Type::Unknown
        | Type::TypeVar(_)
        | Type::Protocol { .. }
        | Type::Intersection(_) => unsupported_schema_type(ty),
    }
}

fn extract_option_schema_type(members: &[Type]) -> Option<IpcSchemaType> {
    if members.len() != 2 || !members.iter().any(|member| matches!(member, Type::None)) {
        return None;
    }
    let payload = members
        .iter()
        .find(|member| !matches!(member, Type::None))?;
    Some(IpcSchemaType::Option(Box::new(extract_ipc_schema_type(
        payload,
    ))))
}

fn unsupported_schema_type(ty: &Type) -> IpcSchemaType {
    IpcSchemaType::Unsupported {
        type_name: ty.display_name(),
    }
}

#[cfg(test)]
mod tests {
    use super::extract_ipc_schema_type;
    use sifr_stdlib::{canonical_schema_descriptor, IpcSchemaDescriptor, IpcSchemaType};
    use sifr_type_system::{FunctionType, Type};

    #[test]
    fn extracts_initial_payload_schema_families() {
        let request = Type::Class {
            name: "EchoRequest".to_string(),
            fields: vec![
                ("message".to_string(), Type::Str),
                (
                    "tags".to_string(),
                    Type::List(Box::new(Type::Union(vec![Type::None, Type::Str]))),
                ),
                (
                    "metadata".to_string(),
                    Type::Dict(Box::new(Type::Str), Box::new(Type::Bytes)),
                ),
                (
                    "outcome".to_string(),
                    Type::Result(Box::new(Type::Bool), Box::new(Type::Str)),
                ),
                (
                    "coords".to_string(),
                    Type::Tuple(vec![Type::Int, Type::Float]),
                ),
            ],
            methods: vec![],
            parent_class: None,
        };

        let descriptor = IpcSchemaDescriptor {
            protocol_schema_version: 1,
            module_path: "demo.ipc".to_string(),
            schema_name: "Echo".to_string(),
            compatible_version_min: 1,
            compatible_version_max: 1,
            request: extract_ipc_schema_type(&request),
            response: extract_ipc_schema_type(&Type::Enum {
                name: "EchoStatus".to_string(),
                variants: vec![
                    ("Accepted".to_string(), Some(1)),
                    ("Rejected".to_string(), Some(2)),
                ],
            }),
            error: extract_ipc_schema_type(&Type::None),
        };

        assert_eq!(
            canonical_schema_descriptor(&descriptor),
            "sifr-ipc-schema-v1\nprotocol_schema_version=1\nmodule=demo.ipc\nschema=Echo\ncompatible=1..1\nrequest=record(EchoRequest{message:str,tags:list(option(str)),metadata:dict(str,bytes),outcome:result(bool,str),coords:tuple(int,float)})\nresponse=enum(EchoStatus{Accepted,Rejected})\nerror=none"
        );
    }

    #[test]
    fn extracts_unsupported_payload_evidence() {
        let process_reader = Type::Class {
            name: "PipeReader".to_string(),
            fields: vec![("_handle".to_string(), Type::Int)],
            methods: vec![],
            parent_class: None,
        };
        let callable = Type::Function(FunctionType::new(
            vec![("value".to_string(), Type::Int)],
            Type::Int,
        ));

        assert_eq!(
            extract_ipc_schema_type(&process_reader),
            IpcSchemaType::Unsupported {
                type_name: "PipeReader".to_string(),
            }
        );
        assert_eq!(
            extract_ipc_schema_type(&callable),
            IpcSchemaType::Unsupported {
                type_name: "function".to_string(),
            }
        );
    }
}
