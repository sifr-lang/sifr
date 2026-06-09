use sifr_stdlib::{
    validate_ipc_payload_type, IpcPayloadEligibilityError, IpcSchemaDescriptor, IpcSchemaField,
    IpcSchemaType, IpcSchemaVariant,
};
use sifr_type_system::Type;
use std::collections::HashSet;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::lower) enum IpcSchemaExtractionError {
    UnsupportedPayload { type_name: String },
}

pub(in crate::lower) fn ipc_schema_descriptor_from_types(
    module_path: &str,
    schema_name: &str,
    request: &Type,
    response: &Type,
    error: &Type,
) -> Result<IpcSchemaDescriptor, IpcSchemaExtractionError> {
    let descriptor = IpcSchemaDescriptor {
        protocol_schema_version: 1,
        module_path: module_path.to_string(),
        schema_name: schema_name.to_string(),
        compatible_version_min: 1,
        compatible_version_max: 1,
        request: ipc_schema_type_from_sifr_type(request),
        response: ipc_schema_type_from_sifr_type(response),
        error: ipc_schema_type_from_sifr_type(error),
    };
    validate_descriptor_payloads(&descriptor)?;
    Ok(descriptor)
}

fn validate_descriptor_payloads(
    descriptor: &IpcSchemaDescriptor,
) -> Result<(), IpcSchemaExtractionError> {
    validate_ipc_payload_type(&descriptor.request)?;
    validate_ipc_payload_type(&descriptor.response)?;
    validate_ipc_payload_type(&descriptor.error)?;
    Ok(())
}

impl From<IpcPayloadEligibilityError> for IpcSchemaExtractionError {
    fn from(value: IpcPayloadEligibilityError) -> Self {
        match value {
            IpcPayloadEligibilityError::UnsupportedPayload { type_name } => {
                Self::UnsupportedPayload { type_name }
            }
        }
    }
}

impl IpcSchemaExtractionError {
    pub(in crate::lower) fn reason(&self) -> String {
        match self {
            Self::UnsupportedPayload { type_name } => {
                format!("`{type_name}` does not have a generated IPC schema")
            }
        }
    }
}

fn ipc_schema_type_from_sifr_type(ty: &Type) -> IpcSchemaType {
    ipc_schema_type_from_sifr_type_inner(ty.resolve_alias(), &mut HashSet::new())
}

fn ipc_schema_type_from_sifr_type_inner(
    ty: &Type,
    visiting: &mut HashSet<String>,
) -> IpcSchemaType {
    match ty {
        Type::Bool | Type::LiteralBool(_) => IpcSchemaType::Bool,
        Type::Int | Type::FixedInt(_) | Type::LiteralInt(_) => IpcSchemaType::Int,
        Type::Float => IpcSchemaType::Float,
        Type::Str | Type::LiteralStr(_) => IpcSchemaType::Str,
        Type::Bytes => IpcSchemaType::Bytes,
        Type::None => IpcSchemaType::None,
        Type::List(inner) => IpcSchemaType::List(Box::new(ipc_schema_type_from_sifr_type_inner(
            inner.resolve_alias(),
            visiting,
        ))),
        Type::Dict(key, value) => {
            if !matches!(key.resolve_alias(), Type::Str) {
                return unsupported(ty);
            }
            IpcSchemaType::DictStr(Box::new(ipc_schema_type_from_sifr_type_inner(
                value.resolve_alias(),
                visiting,
            )))
        }
        Type::Result(ok, err) => IpcSchemaType::Result(
            Box::new(ipc_schema_type_from_sifr_type_inner(
                ok.resolve_alias(),
                visiting,
            )),
            Box::new(ipc_schema_type_from_sifr_type_inner(
                err.resolve_alias(),
                visiting,
            )),
        ),
        Type::Tuple(items) => IpcSchemaType::Tuple(
            items
                .iter()
                .map(|item| ipc_schema_type_from_sifr_type_inner(item.resolve_alias(), visiting))
                .collect(),
        ),
        Type::Union(members) => option_payload_member(members).map_or_else(
            || unsupported(ty),
            |inner| {
                IpcSchemaType::Option(Box::new(ipc_schema_type_from_sifr_type_inner(
                    inner.resolve_alias(),
                    visiting,
                )))
            },
        ),
        Type::Alias { body, .. } => ipc_schema_type_from_sifr_type_inner(body, visiting),
        Type::Newtype { name, inner } => IpcSchemaType::Record {
            name: name.clone(),
            fields: vec![IpcSchemaField {
                name: "value".to_string(),
                ty: ipc_schema_type_from_sifr_type_inner(inner.resolve_alias(), visiting),
            }],
        },
        Type::Class { name, fields, .. } => {
            if is_process_local_or_runtime_resource(name) || !visiting.insert(name.clone()) {
                return unsupported(ty);
            }
            let schema_fields = fields
                .iter()
                .map(|(field_name, field_ty)| IpcSchemaField {
                    name: field_name.clone(),
                    ty: ipc_schema_type_from_sifr_type_inner(field_ty.resolve_alias(), visiting),
                })
                .collect();
            visiting.remove(name);
            IpcSchemaType::Record {
                name: name.clone(),
                fields: schema_fields,
            }
        }
        Type::Enum { name, variants } => IpcSchemaType::Enum {
            name: name.clone(),
            variants: variants
                .iter()
                .map(|(variant_name, _)| IpcSchemaVariant {
                    name: variant_name.clone(),
                    payload: None,
                })
                .collect(),
        },
        _ => unsupported(ty),
    }
}

fn option_payload_member(members: &[Type]) -> Option<&Type> {
    if members.len() != 2 || !members.iter().any(|member| matches!(member, Type::None)) {
        return None;
    }
    members.iter().find(|member| !matches!(member, Type::None))
}

fn unsupported(ty: &Type) -> IpcSchemaType {
    IpcSchemaType::Unsupported {
        type_name: ty.display_name(),
    }
}

fn is_process_local_or_runtime_resource(name: &str) -> bool {
    matches!(
        public_type_name(name),
        "Child"
            | "AsyncChild"
            | "ProcessHandle"
            | "PipeReader"
            | "PipeWriter"
            | "AsyncPipeReader"
            | "AsyncPipeWriter"
            | "Lock"
            | "RwLock"
            | "Semaphore"
            | "Notify"
            | "Shared"
            | "LockGuard"
            | "RwLockReadGuard"
            | "RwLockWriteGuard"
            | "SemaphorePermit"
            | "Channel"
            | "ChannelSender"
            | "ChannelReceiver"
            | "Context"
            | "ContextKey"
    )
}

fn public_type_name(name: &str) -> &str {
    name.rsplit_once("::").map_or(name, |(_, tail)| tail)
}

#[cfg(test)]
mod tests {
    use super::{
        ipc_schema_descriptor_from_types, IpcSchemaExtractionError, IpcSchemaType, IpcSchemaVariant,
    };
    use sifr_stdlib::{canonical_schema_descriptor, schema_hash_hex_v1};
    use sifr_type_system::{FunctionType, Type};

    #[test]
    fn extracts_stable_schema_descriptor_from_sifr_type_graphs() {
        let request = Type::Class {
            name: "EchoRequest".to_string(),
            fields: vec![
                ("message".to_string(), Type::Str),
                ("attempts".to_string(), Type::List(Box::new(Type::Int))),
                (
                    "metadata".to_string(),
                    Type::Dict(
                        Box::new(Type::Str),
                        Box::new(Type::Union(vec![Type::None, Type::Bytes])),
                    ),
                ),
            ],
            methods: vec![],
            parent_class: None,
        };
        let response = Type::Class {
            name: "EchoResponse".to_string(),
            fields: vec![("accepted".to_string(), Type::Bool)],
            methods: vec![],
            parent_class: None,
        };
        let error = Type::Enum {
            name: "EchoError".to_string(),
            variants: vec![
                ("Rejected".to_string(), Some(1)),
                ("Closed".to_string(), Some(2)),
            ],
        };

        let descriptor =
            ipc_schema_descriptor_from_types("demo.worker", "Echo", &request, &response, &error)
                .expect("schema should be extracted");

        assert_eq!(
            canonical_schema_descriptor(&descriptor),
            "sifr-ipc-schema-v1\nprotocol_schema_version=1\nmodule=demo.worker\nschema=Echo\ncompatible=1..1\nrequest=record(EchoRequest{message:str,attempts:list(int),metadata:dict(str,option(bytes))})\nresponse=record(EchoResponse{accepted:bool})\nerror=enum(EchoError{Rejected,Closed})"
        );
        assert_eq!(schema_hash_hex_v1(&descriptor).len(), 32);
    }

    #[test]
    fn extracts_newtypes_and_payloadless_enums() {
        let request = Type::Newtype {
            name: "WorkerId".to_string(),
            inner: Box::new(Type::Int),
        };
        let response = Type::Enum {
            name: "WorkerState".to_string(),
            variants: vec![("Idle".to_string(), Some(0)), ("Busy".to_string(), Some(1))],
        };

        let descriptor = ipc_schema_descriptor_from_types(
            "demo.worker",
            "WorkerStatus",
            &request,
            &response,
            &Type::None,
        )
        .expect("schema should be extracted");

        assert!(matches!(
            descriptor.request,
            IpcSchemaType::Record { ref fields, .. } if fields.len() == 1
        ));
        assert!(matches!(
            descriptor.response,
            IpcSchemaType::Enum { ref variants, .. }
                if variants == &vec![
                    IpcSchemaVariant { name: "Idle".to_string(), payload: None },
                    IpcSchemaVariant { name: "Busy".to_string(), payload: None },
                ]
        ));
    }

    #[test]
    fn rejects_process_local_resources_inside_generated_records() {
        let request = Type::Class {
            name: "Request".to_string(),
            fields: vec![(
                "pipe".to_string(),
                Type::Class {
                    name: "PipeReader".to_string(),
                    fields: vec![("_handle".to_string(), Type::Int)],
                    methods: vec![],
                    parent_class: None,
                },
            )],
            methods: vec![],
            parent_class: None,
        };

        assert_eq!(
            ipc_schema_descriptor_from_types(
                "demo.worker",
                "BadRequest",
                &request,
                &Type::None,
                &Type::None,
            ),
            Err(IpcSchemaExtractionError::UnsupportedPayload {
                type_name: "PipeReader".to_string(),
            })
        );
    }

    #[test]
    fn rejects_schema_less_dynamic_or_unordered_payloads() {
        let cases = [
            Type::Dict(Box::new(Type::Int), Box::new(Type::Str)),
            Type::Set(Box::new(Type::Int)),
            Type::Union(vec![Type::Int, Type::Str]),
            Type::Any,
            Type::Function(FunctionType::new(
                vec![("value".to_string(), Type::Int)],
                Type::Int,
            )),
        ];

        for request in cases {
            assert!(matches!(
                ipc_schema_descriptor_from_types(
                    "demo.worker",
                    "Rejected",
                    &request,
                    &Type::None,
                    &Type::None,
                ),
                Err(IpcSchemaExtractionError::UnsupportedPayload { .. })
            ));
        }
    }
}
