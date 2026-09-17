use super::{Decode, Decoder, Result, wire};
use sifr_type_system::Type;
impl Decode<wire::Ref<wire::Type>> for Type {
    fn decode(reference: &wire::Ref<wire::Type>, cx: &mut Decoder) -> Result<Self> {
        cx.record(*reference, |value, cx| {
            Ok(match value {
                wire::Type::Int => Self::Int,
                wire::Type::FixedInt(v0) => Self::FixedInt(Decode::decode(v0, cx)?),
                wire::Type::Float => Self::Float,
                wire::Type::Bool => Self::Bool,
                wire::Type::Str => Self::Str,
                wire::Type::Bytes => Self::Bytes,
                wire::Type::None => Self::None,
                wire::Type::Function(v0) => Self::Function(Decode::decode(v0, cx)?),
                wire::Type::AsyncFunction(v0) => Self::AsyncFunction(Decode::decode(v0, cx)?),
                wire::Type::Coroutine(v0, v1) => {
                    Self::Coroutine(Decode::decode(v0, cx)?, Decode::decode(v1, cx)?)
                }
                wire::Type::Task(v0, v1) => {
                    Self::Task(Decode::decode(v0, cx)?, Decode::decode(v1, cx)?)
                }
                wire::Type::TaskResult(v0, v1) => {
                    Self::TaskResult(Decode::decode(v0, cx)?, Decode::decode(v1, cx)?)
                }
                wire::Type::Failure(v0) => Self::Failure(Decode::decode(v0, cx)?),
                wire::Type::TimeoutResult(v0) => Self::TimeoutResult(Decode::decode(v0, cx)?),
                wire::Type::Select2(v0, v1) => {
                    Self::Select2(Decode::decode(v0, cx)?, Decode::decode(v1, cx)?)
                }
                wire::Type::BlockingTask(v0, v1) => {
                    Self::BlockingTask(Decode::decode(v0, cx)?, Decode::decode(v1, cx)?)
                }
                wire::Type::JoinSet(v0, v1) => {
                    Self::JoinSet(Decode::decode(v0, cx)?, Decode::decode(v1, cx)?)
                }
                wire::Type::Awaitable(v0) => Self::Awaitable(Decode::decode(v0, cx)?),
                wire::Type::AsyncIterator(v0, v1) => {
                    Self::AsyncIterator(Decode::decode(v0, cx)?, Decode::decode(v1, cx)?)
                }
                wire::Type::AsyncGenerator(v0, v1) => {
                    Self::AsyncGenerator(Decode::decode(v0, cx)?, Decode::decode(v1, cx)?)
                }
                wire::Type::PythonBuffer(v0) => Self::PythonBuffer(Decode::decode(v0, cx)?),
                wire::Type::PythonArrow(v0) => Self::PythonArrow(Decode::decode(v0, cx)?),
                wire::Type::PythonDlpackTensor(v0) => {
                    Self::PythonDlpackTensor(Decode::decode(v0, cx)?)
                }
                wire::Type::PythonDlpackStream => Self::PythonDlpackStream,
                wire::Type::List(v0) => Self::List(Decode::decode(v0, cx)?),
                wire::Type::Dict(v0, v1) => {
                    Self::Dict(Decode::decode(v0, cx)?, Decode::decode(v1, cx)?)
                }
                wire::Type::Set(v0) => Self::Set(Decode::decode(v0, cx)?),
                wire::Type::Tuple(v0) => Self::Tuple(Decode::decode(v0, cx)?),
                wire::Type::Template(v0) => Self::Template(Decode::decode(v0, cx)?),
                wire::Type::StructuralRecord(v0) => Self::StructuralRecord(Decode::decode(v0, cx)?),
                wire::Type::Range => Self::Range,
                wire::Type::Iterable(v0) => Self::Iterable(Decode::decode(v0, cx)?),
                wire::Type::Iterator(v0) => Self::Iterator(Decode::decode(v0, cx)?),
                wire::Type::Any => Self::Any,
                wire::Type::Never => Self::Never,
                wire::Type::Union(v0) => Self::Union(Decode::decode(v0, cx)?),
                wire::Type::Intersection(v0) => Self::Intersection(Decode::decode(v0, cx)?),
                wire::Type::LiteralInt(v0) => Self::LiteralInt(Decode::decode(v0, cx)?),
                wire::Type::LiteralStr(v0) => Self::LiteralStr(Decode::decode(v0, cx)?),
                wire::Type::LiteralBool(v0) => Self::LiteralBool(Decode::decode(v0, cx)?),
                wire::Type::Unknown => Self::Unknown,
                wire::Type::Result(v0, v1) => {
                    Self::Result(Decode::decode(v0, cx)?, Decode::decode(v1, cx)?)
                }
                wire::Type::Callable(v0, v1, v2) => Self::Callable(
                    Decode::decode(v0, cx)?,
                    Decode::decode(v1, cx)?,
                    Decode::decode(v2, cx)?,
                ),
                wire::Type::AsyncCallable(v0, v1, v2) => Self::AsyncCallable(
                    Decode::decode(v0, cx)?,
                    Decode::decode(v1, cx)?,
                    Decode::decode(v2, cx)?,
                ),
                wire::Type::Decimal => Self::Decimal,
                wire::Type::BigDecimal => Self::BigDecimal,
                wire::Type::TypeVar { binder, slot } => {
                    let binder = cx.store.get(*binder)?;
                    let (name, _) = binder
                        .parameters
                        .get(*slot as usize)
                        .ok_or_else(|| wire::MetadataError("invalid binder slot".into()))?;
                    Self::TypeVar(Decode::decode(name, cx)?)
                }
                wire::Type::Alias {
                    name,
                    type_args,
                    body,
                    ..
                } => Self::Alias {
                    name: Decode::decode(name, cx)?,
                    type_args: Decode::decode(type_args, cx)?,
                    body: Box::new(match body {
                        Some(v) => Decode::decode(v, cx)?,
                        None => Type::Unknown,
                    }),
                },
                wire::Type::Class {
                    view, type_args, ..
                } => {
                    let v = cx.nominal(*view)?;
                    Self::Class {
                        identity: v.identity.clone(),
                        name: v.name.clone(),
                        type_args: Decode::decode(type_args, cx)?,
                        fields: v.fields.clone(),
                        methods: v.methods.clone(),
                        parent_class: v.parent_class.clone(),
                    }
                }
                wire::Type::Protocol { view, .. } => {
                    let v = cx.nominal(*view)?;
                    Self::Protocol {
                        identity: v.identity.clone(),
                        name: v.name.clone(),
                        methods: v.methods.clone(),
                    }
                }
                wire::Type::Newtype { view, .. } => {
                    let v = cx.nominal(*view)?;
                    Self::Newtype {
                        identity: v.identity.clone(),
                        name: v.name.clone(),
                        inner: Box::new(v.newtype_inner.clone().ok_or_else(|| {
                            wire::MetadataError("newtype missing inner type".into())
                        })?),
                    }
                }
                wire::Type::Enum { view, .. } => {
                    let v = cx.nominal(*view)?;
                    Self::Enum {
                        identity: v.identity.clone(),
                        name: v.name.clone(),
                        variants: v.enum_variants.clone(),
                    }
                }
            })
        })
    }
}
impl Decode<wire::Ref<wire::ParamConvention>> for sifr_type_system::ParamConvention {
    fn decode(r: &wire::Ref<wire::ParamConvention>, cx: &mut Decoder) -> Result<Self> {
        let v = cx.store.get(*r)?;
        Ok(Self::new(
            Decode::decode(&v.ownership, cx)?,
            Decode::decode(&v.mutability, cx)?,
        ))
    }
}
impl Decode<wire::Ref<wire::StructuralRecordField>> for sifr_type_system::StructuralRecordField {
    fn decode(r: &wire::Ref<wire::StructuralRecordField>, cx: &mut Decoder) -> Result<Self> {
        let v = cx.store.get(*r)?;
        Ok(Self::new(
            Decode::decode(&v.name, cx)?,
            Decode::decode(&v.ty, cx)?,
        ))
    }
}
impl Decode<wire::Ref<wire::StructuralRecordType>> for sifr_type_system::StructuralRecordType {
    fn decode(r: &wire::Ref<wire::StructuralRecordType>, cx: &mut Decoder) -> Result<Self> {
        let v = cx.store.get(*r)?;
        let fields: Vec<sifr_type_system::StructuralRecordField> =
            Decode::decode(&v.canonical_fields, cx)?;
        let order: Vec<String> = Decode::decode(&v.source_order, cx)?;
        let fields = order
            .into_iter()
            .map(|name| {
                let field = fields
                    .iter()
                    .find(|f| f.name() == name)
                    .ok_or_else(|| wire::MetadataError("structural field order mismatch".into()))?;
                Ok((name, field.ty().clone()))
            })
            .collect::<Result<_>>()?;
        Ok(Self::new(fields))
    }
}
impl Decode<wire::Ref<wire::TextRange>> for ruff_text_size::TextRange {
    fn decode(r: &wire::Ref<wire::TextRange>, cx: &mut Decoder) -> Result<Self> {
        let v = cx.store.get(*r)?;
        Ok(Self::new(v.start.into(), v.end.into()))
    }
}
impl Decode<wire::Ref<wire::BindingId>> for sifr_ir::BindingId {
    fn decode(r: &wire::Ref<wire::BindingId>, cx: &mut Decoder) -> Result<Self> {
        let v = cx.store.get(*r)?;
        Ok(Self(v.slot))
    }
}
impl Decode<wire::Ref<wire::SourceOriginId>> for sifr_ir::SourceOriginId {
    fn decode(r: &wire::Ref<wire::SourceOriginId>, cx: &mut Decoder) -> Result<Self> {
        let v = cx.store.get(*r)?;
        let location = cx.store.get(v.location)?;
        Ok(Self::new(location.source.id(), v.local_ordinal))
    }
}
