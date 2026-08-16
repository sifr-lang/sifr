pub(crate) mod generated_types;
mod type_shapes;

use type_shapes::{
    bridge_dict_type, bridge_list_type, bridge_tuple_type, bridge_union_type, combine_generic_type,
    simple_type,
};

use crate::rust_interop_bridge_callback_contract::{
    bridge_call_scoped_callback_type, bridge_threadsafe_callback_type, CallbackSignature,
};
pub(crate) use crate::rust_interop_bridge_contract_serialization::push_bridge_contract_plan;
use crate::rust_interop_bridge_panic_contract::{
    recoverable_panic_bridge_error, rust_bridge_panic_error_contract,
};
use crate::rust_interop_plan::{RustInteropOwner, RustInteropPlanDeclaration};
use generated_types::{
    absolute_runtime_target, bridge_type_definition_module, class_bridge_declaration_name,
    class_bridge_definition_module, generated_bridge_type_path, generated_class_bridge_type_path,
    is_generated_bridge_type_path, opaque_rust_type_path, opaque_type_definition,
    GeneratedTypeCollector,
};
use sifr_ir::HirModule;
use sifr_type_system::{ParamConvention, ParamOwnership, Type};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RustBridgeContractPlan {
    pub signatures: Vec<RustBridgeSignatureContract>,
    pub generated_types: Vec<RustGeneratedBridgeType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustBridgeSignatureContract {
    pub canonical_target_path: String,
    pub module_name: Option<String>,
    pub owner: RustInteropOwner,
    pub params: Vec<RustBridgeParamContract>,
    pub return_type: RustBridgeTypeContract,
    pub structural_type_params: Vec<String>,
    pub static_program_type_params: Vec<String>,
    pub method_slot_contract: Option<RustBridgeMethodSlotContract>,
    pub panic_error: RustBridgePanicErrorContract,
    pub span: ruff_text_size::TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustBridgeMethodSlotContract {
    pub owner_type_param: String,
    pub context_type_param: String,
    pub context_mutable: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum RustBridgePanicErrorContract {
    #[default]
    None,
    WrapperOnly,
    OrdinaryAndWrapper,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustBridgeParamContract {
    pub name: String,
    pub convention: RustBridgeParamConvention,
    pub ty: RustBridgeTypeContract,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustBridgeParamConvention {
    Borrow,
    MutableBorrow,
    Own,
    OwnMutable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustBridgeTypeContract {
    pub sifr_type: String,
    pub rust_borrowed_type: Option<String>,
    pub rust_owned_type: Option<String>,
    pub rust_return_type: Option<String>,
    pub kind: RustBridgeTypeKind,
    pub unsupported_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustBridgeTypeKind {
    Bool,
    FixedInt,
    ExactInt,
    Float64,
    String,
    Bytes,
    List,
    Dict,
    Option,
    Tuple,
    Result,
    GeneratedRecord,
    GeneratedEnum,
    GeneratedError,
    OpaqueHandle,
    Callback,
    CallScopedCallback,
    StructuralTypeParam,
    None,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RustGeneratedBridgeType {
    pub module_name: Option<String>,
    pub name: String,
    pub rust_type_path: String,
    pub kind: RustGeneratedBridgeTypeKind,
    pub fields: Vec<RustGeneratedBridgeField>,
    pub variants: Vec<RustGeneratedBridgeVariant>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RustGeneratedBridgeTypeKind {
    Record,
    ClosedEnum,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RustGeneratedBridgeField {
    pub name: String,
    pub sifr_type: String,
    pub rust_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RustGeneratedBridgeVariant {
    pub name: String,
    pub discriminant: u32,
}

pub(crate) fn bridge_contract_plan_for_named_modules<'a>(
    modules: impl IntoIterator<Item = (Option<&'a str>, &'a HirModule)>,
    declarations: &[RustInteropPlanDeclaration],
) -> RustBridgeContractPlan {
    let module_catalogs = modules
        .into_iter()
        .map(|(module_name, module)| (module_name.map(str::to_string), ModuleCatalog::new(module)))
        .collect::<BTreeMap<_, _>>();
    let mut generated_types = GeneratedTypeCollector::default();
    let mut signatures = Vec::new();
    let callback_targets = declarations
        .iter()
        .filter(|declaration| {
            declaration.declaration.kind == sifr_ir::RustInteropDecoratorKind::Callback
        })
        .map(RustInteropPlanDeclaration::canonical_sifr_target_path)
        .collect::<BTreeSet<_>>();
    for declaration in declarations {
        if matches!(
            declaration.declaration.kind,
            sifr_ir::RustInteropDecoratorKind::Callback
                | sifr_ir::RustInteropDecoratorKind::Structural
        ) {
            continue;
        }
        if let Some(signature) = signature_contract(
            declaration,
            &module_catalogs,
            &mut generated_types,
            &callback_targets,
        ) {
            signatures.push(signature);
        }
    }

    RustBridgeContractPlan {
        signatures,
        generated_types: generated_types.into_types(),
    }
}

fn signature_contract(
    declaration: &RustInteropPlanDeclaration,
    module_catalogs: &BTreeMap<Option<String>, ModuleCatalog>,
    generated_types: &mut GeneratedTypeCollector,
    callback_targets: &BTreeSet<String>,
) -> Option<RustBridgeSignatureContract> {
    let module_name = declaration.module_name.clone();
    let catalog = module_catalogs.get(&module_name);
    let (function, receiver) = match &declaration.owner {
        RustInteropOwner::Function { name } => (
            catalog.and_then(|catalog| catalog.functions.get(name)),
            None,
        ),
        RustInteropOwner::Method { class_name, name } => {
            let function = catalog
                .and_then(|catalog| catalog.methods.get(&(class_name.clone(), name.clone())));
            let receiver = catalog
                .and_then(|catalog| catalog.opaque_classes.get(class_name))
                .filter(|_| {
                    function.is_some_and(|function| {
                        function.method_kind == Some(sifr_ir::MethodKind::Regular)
                    })
                })
                .filter(|_| {
                    declaration
                        .declaration
                        .target
                        .as_ref()
                        .and_then(|target| target.segments.first())
                        .is_none_or(|root| root != "Self")
                })
                .map(|target| RustBridgeParamContract {
                    name: "self".to_string(),
                    convention: if function.is_some_and(|function| function.consumes_receiver) {
                        RustBridgeParamConvention::Own
                    } else {
                        RustBridgeParamConvention::Borrow
                    },
                    ty: opaque_handle_type(class_name, target),
                });
            (function, receiver)
        }
        RustInteropOwner::Class { .. } => (None, None),
    };
    let function = function?;
    let structural_type_params = function.structural_type_params.as_slice();
    let params = receiver
        .into_iter()
        .chain(function.params.iter().map(|param| RustBridgeParamContract {
            name: param.name.clone(),
            convention: bridge_param_convention(param.convention),
            ty: bridge_type_contract(
                &param.ty,
                module_name.as_ref(),
                module_catalogs,
                catalog,
                generated_types,
                BridgeTypePosition::Parameter(
                    if callback_targets.contains(&declaration.canonical_sifr_target_path()) {
                        CallbackParameterMode::Threadsafe
                    } else {
                        CallbackParameterMode::CallScoped
                    },
                ),
                structural_type_params,
            ),
        }))
        .collect::<Vec<_>>();
    let return_type = bridge_type_contract(
        &function.return_type,
        module_name.as_ref(),
        module_catalogs,
        catalog,
        generated_types,
        BridgeTypePosition::Return,
        structural_type_params,
    );
    let panic_error = rust_bridge_panic_error_contract(&function.return_type);
    Some(RustBridgeSignatureContract {
        canonical_target_path: declaration.canonical_sifr_target_path(),
        module_name,
        owner: declaration.owner.clone(),
        params,
        return_type,
        structural_type_params: function.structural_type_params.clone(),
        static_program_type_params: function.static_program_type_params.clone(),
        method_slot_contract: function.method_slot_contract.clone(),
        panic_error,
        span: declaration.declaration.span,
    })
}

#[derive(Clone)]
struct ModuleFunction {
    params: Vec<sifr_ir::HirParam>,
    return_type: Type,
    method_kind: Option<sifr_ir::MethodKind>,
    consumes_receiver: bool,
    structural_type_params: Vec<String>,
    static_program_type_params: Vec<String>,
    method_slot_contract: Option<RustBridgeMethodSlotContract>,
}

fn method_slot_contract(
    module: &HirModule,
    function: &sifr_ir::HirFunction,
) -> Option<RustBridgeMethodSlotContract> {
    let bounds = module.type_param_bounds.get(&function.name)?;
    let owner_type_param = function
        .type_params
        .iter()
        .find(|type_param| {
            bounds
                .get(*type_param)
                .is_some_and(|bounds| bounds.as_slice() == ["MethodSlots"])
        })?
        .clone();
    let context_type_param = function
        .type_params
        .iter()
        .find(|type_param| {
            bounds
                .get(*type_param)
                .is_some_and(|bounds| bounds.as_slice() == ["Context"])
        })?
        .clone();
    let context_mutable = function.params.iter().find_map(|param| {
        matches!(param.ty.resolve_alias(), Type::TypeVar(name) if name == &context_type_param)
            .then_some(param.convention.is_mut_borrow())
    })?;
    Some(RustBridgeMethodSlotContract {
        owner_type_param,
        context_type_param,
        context_mutable,
    })
}

pub(crate) struct ModuleCatalog {
    functions: BTreeMap<String, ModuleFunction>,
    methods: BTreeMap<(String, String), ModuleFunction>,
    opaque_classes: BTreeMap<String, String>,
    error_classes: BTreeSet<String>,
    record_classes: BTreeSet<String>,
    enum_classes: BTreeSet<String>,
}

impl ModuleCatalog {
    fn new(module: &HirModule) -> Self {
        let mut functions = BTreeMap::new();
        let mut methods = BTreeMap::new();
        let mut opaque_classes = BTreeMap::new();
        let mut error_classes = BTreeSet::new();
        let mut record_classes = BTreeSet::new();
        let mut enum_classes = BTreeSet::new();
        for function in &module.functions {
            let has_structural_declaration = function.rust_interop.iter().any(|declaration| {
                declaration.kind == sifr_ir::RustInteropDecoratorKind::Structural
            });
            let structural_type_params = if has_structural_declaration {
                function
                    .type_params
                    .iter()
                    .filter(|type_param| {
                        module
                            .type_param_bounds
                            .get(&function.name)
                            .and_then(|bounds| bounds.get(*type_param))
                            .is_some_and(|bounds| {
                                matches!(
                                    bounds.as_slice(),
                                    [bound]
                                        if matches!(
                                            bound.as_str(),
                                            "Structural" | "StaticProgram" | "MethodSlots" | "Context"
                                        )
                                )
                            })
                    })
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            };
            functions.insert(
                function.name.clone(),
                ModuleFunction {
                    params: function.params.clone(),
                    return_type: function.return_type.clone(),
                    method_kind: None,
                    consumes_receiver: false,
                    structural_type_params,
                    static_program_type_params: function
                        .type_params
                        .iter()
                        .filter(|type_param| {
                            module
                                .type_param_bounds
                                .get(&function.name)
                                .and_then(|bounds| bounds.get(*type_param))
                            .is_some_and(|bounds| {
                                matches!(bounds.as_slice(), [bound] if bound == "StaticProgram" || bound == "MethodSlots")
                            })
                        })
                        .cloned()
                        .collect(),
                    method_slot_contract: method_slot_contract(module, function),
                },
            );
        }
        for class in &module.classes {
            if class
                .rust_interop
                .iter()
                .any(|declaration| declaration.kind == sifr_ir::RustInteropDecoratorKind::Opaque)
            {
                if let Some(target) = opaque_rust_type_path(class) {
                    opaque_classes.insert(class.name.clone(), target);
                }
            }
            if class.is_enum() {
                enum_classes.insert(class.name.clone());
            } else if !class
                .rust_interop
                .iter()
                .any(|declaration| declaration.kind == sifr_ir::RustInteropDecoratorKind::Opaque)
            {
                record_classes.insert(class.name.clone());
            }
            if class.is_error_type {
                error_classes.insert(class.name.clone());
            }
            for method in &class.methods {
                methods.insert(
                    (class.name.clone(), method.name.clone()),
                    ModuleFunction {
                        params: method.params.clone(),
                        return_type: method.return_type.clone(),
                        method_kind: Some(method.method_kind),
                        consumes_receiver: method
                            .rust_interop
                            .first()
                            .is_some_and(|declaration| declaration.consumes_receiver),
                        structural_type_params: Vec::new(),
                        static_program_type_params: Vec::new(),
                        method_slot_contract: None,
                    },
                );
            }
        }
        Self {
            functions,
            methods,
            opaque_classes,
            error_classes,
            record_classes,
            enum_classes,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum BridgeTypePosition {
    Parameter(CallbackParameterMode),
    Return,
}

#[derive(Clone, Copy)]
pub(crate) enum CallbackParameterMode {
    Nested,
    CallScoped,
    Threadsafe,
}

impl BridgeTypePosition {
    fn nested(self) -> Self {
        match self {
            Self::Parameter(_) => Self::Parameter(CallbackParameterMode::Nested),
            Self::Return => Self::Return,
        }
    }
}

pub(crate) fn bridge_type_contract(
    ty: &Type,
    module_name: Option<&String>,
    module_catalogs: &BTreeMap<Option<String>, ModuleCatalog>,
    catalog: Option<&ModuleCatalog>,
    generated_types: &mut GeneratedTypeCollector,
    position: BridgeTypePosition,
    structural_type_params: &[String],
) -> RustBridgeTypeContract {
    let resolved = ty.resolve_alias();
    match resolved {
        Type::Bool => simple_type("bool", "bool", RustBridgeTypeKind::Bool),
        Type::FixedInt(fixed) => simple_type(
            fixed.source_name(),
            fixed.rust_name(),
            RustBridgeTypeKind::FixedInt,
        ),
        Type::Int => RustBridgeTypeContract {
            sifr_type: "int".to_string(),
            rust_borrowed_type: Some("&::sifr_runtime::interop::SifrIntBridge".to_string()),
            rust_owned_type: Some("::sifr_runtime::interop::SifrIntBridge".to_string()),
            rust_return_type: Some("::sifr_runtime::interop::SifrIntBridge".to_string()),
            kind: RustBridgeTypeKind::ExactInt,
            unsupported_reason: None,
        },
        Type::Float => simple_type("float", "f64", RustBridgeTypeKind::Float64),
        Type::Str => RustBridgeTypeContract {
            sifr_type: "str".to_string(),
            rust_borrowed_type: Some("&str".to_string()),
            rust_owned_type: Some("String".to_string()),
            rust_return_type: Some("String".to_string()),
            kind: RustBridgeTypeKind::String,
            unsupported_reason: None,
        },
        Type::Bytes => RustBridgeTypeContract {
            sifr_type: "bytes".to_string(),
            rust_borrowed_type: Some("&[u8]".to_string()),
            rust_owned_type: Some("Vec<u8>".to_string()),
            rust_return_type: Some("Vec<u8>".to_string()),
            kind: RustBridgeTypeKind::Bytes,
            unsupported_reason: None,
        },
        Type::None => simple_type("None", "()", RustBridgeTypeKind::None),
        Type::List(inner) => bridge_list_type(
            inner,
            module_name,
            module_catalogs,
            catalog,
            generated_types,
            position,
            structural_type_params,
        ),
        Type::Dict(key, value) => bridge_dict_type(
            (key, value),
            module_name,
            module_catalogs,
            catalog,
            generated_types,
            position,
            structural_type_params,
        ),
        Type::Union(members) => bridge_union_type(
            members,
            module_name,
            module_catalogs,
            catalog,
            generated_types,
            position,
            structural_type_params,
        ),
        Type::Tuple(items) => bridge_tuple_type(
            items,
            ty.display_name(),
            module_name,
            module_catalogs,
        ),
        Type::Result(ok, err) => match position {
            BridgeTypePosition::Parameter(_) => unsupported_type(
                ty,
                "Result[T, E] is bridge-compatible only as a Rust return type",
            ),
            BridgeTypePosition::Return => {
                let ok_ty = bridge_type_contract(
                    ok,
                    module_name,
                    module_catalogs,
                    catalog,
                    generated_types,
                    BridgeTypePosition::Return,
                    structural_type_params,
                );
                let bridge_error = match recoverable_panic_bridge_error(err) {
                    Ok(Some(ordinary_error)) => ordinary_error,
                    Ok(None) => err,
                    Err(reason) => {
                        return unsupported_type(err, reason);
                    }
                };
                let err_ty = bridge_type_contract(
                    bridge_error,
                    module_name,
                    module_catalogs,
                    catalog,
                    generated_types,
                    BridgeTypePosition::Return,
                    structural_type_params,
                );
                combine_generic_type(
                    "Result",
                    ty.display_name(),
                    RustBridgeTypeKind::Result,
                    &[ok_ty, err_ty],
                )
            }
        },
        class_type @ Type::Class {
            name, parent_class, ..
        } => {
            let opaque_target = opaque_type_definition(name, module_name, module_catalogs);
            if let Ok(Some(target)) = opaque_target {
                opaque_handle_type(name, &target)
            } else if let Err(reason) = opaque_target {
                unsupported_type(ty, &reason)
            } else {
                let declaration_module =
                    match class_bridge_definition_module(class_type, module_name, module_catalogs) {
                        Ok(module_name) => module_name,
                        Err(reason) => return unsupported_type(ty, &reason),
                    };
                let is_error = parent_class.as_deref() == Some("Error")
                    || module_catalogs
                        .get(&declaration_module)
                        .is_some_and(|catalog| {
                            catalog
                                .error_classes
                                .contains(class_bridge_declaration_name(class_type))
                        });
                generated_types.insert_record(
                    declaration_module.as_ref(),
                    class_type,
                    is_error,
                    module_catalogs,
                );
                let path =
                    generated_class_bridge_type_path(declaration_module.as_ref(), class_type);
                RustBridgeTypeContract {
                    sifr_type: ty.display_name(),
                    rust_borrowed_type: Some(path.clone()),
                    rust_owned_type: Some(path.clone()),
                    rust_return_type: Some(path),
                    kind: if is_error {
                        RustBridgeTypeKind::GeneratedError
                    } else {
                        RustBridgeTypeKind::GeneratedRecord
                    },
                    unsupported_reason: None,
                }
            }
        }
        Type::Enum { name, variants, .. } => {
            let declaration_module =
                match bridge_type_definition_module(name, module_name, module_catalogs, true) {
                    Ok(module_name) => module_name,
                    Err(reason) => return unsupported_type(ty, &reason),
                };
            if let Err(reason) = generated_types.insert_enum(declaration_module.as_ref(), name, variants) {
                return unsupported_type(ty, &reason);
            }
            let path = generated_bridge_type_path(declaration_module.as_ref(), name);
            RustBridgeTypeContract {
                sifr_type: ty.display_name(),
                rust_borrowed_type: Some(path.clone()),
                rust_owned_type: Some(path.clone()),
                rust_return_type: Some(path),
                kind: RustBridgeTypeKind::GeneratedEnum,
                unsupported_reason: None,
            }
        }
        Type::Callable(params, conventions, result)
            if matches!(
                position,
                BridgeTypePosition::Parameter(CallbackParameterMode::Threadsafe)
            ) =>
        {
            bridge_threadsafe_callback_type(
                CallbackSignature {
                    callable: ty,
                    params,
                    conventions,
                    result,
                    structural_type_params,
                },
                module_name,
                module_catalogs,
                catalog,
                generated_types,
            )
        }
        Type::Callable(params, conventions, result)
            if matches!(
                position,
                BridgeTypePosition::Parameter(CallbackParameterMode::CallScoped)
            ) =>
        {
            bridge_call_scoped_callback_type(
                CallbackSignature {
                    callable: ty,
                    params,
                    conventions,
                    result,
                    structural_type_params,
                },
                module_name,
                module_catalogs,
                catalog,
                generated_types,
            )
        }
        Type::Callable(..) => {
            unsupported_type(ty, "call-scoped callbacks are valid only as top-level parameters")
        }
        Type::AsyncCallable(..) => unsupported_type(
            ty,
            "async callbacks require explicit callback contract support before they are bridge-compatible",
        ),
        Type::Set(_) => unsupported_type(ty, "set[T] is not a supported Rust bridge container"),
        Type::Any | Type::Unknown => {
            unsupported_type(ty, "dynamic Any/Unknown values are not Rust bridge-compatible")
        }
        Type::Never => unsupported_type(ty, "Never is not a Rust bridge value type"),
        Type::TypeVar(name) if structural_type_params.contains(name) => {
            RustBridgeTypeContract {
                sifr_type: name.clone(),
                rust_borrowed_type: Some(format!("&{name}")),
                rust_owned_type: Some(name.clone()),
                rust_return_type: Some(name.clone()),
                kind: RustBridgeTypeKind::StructuralTypeParam,
                unsupported_reason: None,
            }
        }
        Type::TypeVar(_) => unsupported_type(ty, "unconstrained generics are not Rust bridge-compatible"),
        Type::Function(_) | Type::AsyncFunction(_) => {
            unsupported_type(ty, "function values are not Rust bridge-compatible")
        }
        Type::Range
        | Type::PythonBuffer(_)
        | Type::PythonArrow(_)
        | Type::PythonDlpackTensor(_)
        | Type::PythonDlpackStream
        | Type::Iterable(_)
        | Type::Iterator(_)
        | Type::Coroutine(_, _)
        | Type::Task(_, _)
        | Type::TaskResult(_, _)
        | Type::Failure(_)
        | Type::TimeoutResult(_)
        | Type::Select2(_, _)
        | Type::BlockingTask(_, _)
        | Type::JoinSet(_, _)
        | Type::Awaitable(_)
        | Type::AsyncIterator(_, _)
        | Type::AsyncGenerator(_, _)
        | Type::Intersection(_)
        | Type::LiteralInt(_)
        | Type::LiteralStr(_)
        | Type::LiteralBool(_)
        | Type::Protocol { .. }
        | Type::Newtype { .. }
        | Type::BigInt
        | Type::Decimal
        | Type::BigDecimal => unsupported_type(
            ty,
            "type is outside the initial Rust bridge-compatible contract",
        ),
        Type::Alias { .. } => unreachable!("resolved aliases must not remain aliases"),
    }
}

fn opaque_handle_type(name: &str, target: &str) -> RustBridgeTypeContract {
    let rust_type = rust_opaque_handle_type(target);
    RustBridgeTypeContract {
        sifr_type: name.to_string(),
        rust_borrowed_type: Some(format!("&{rust_type}")),
        rust_owned_type: Some(rust_type.clone()),
        rust_return_type: Some(rust_type),
        kind: RustBridgeTypeKind::OpaqueHandle,
        unsupported_reason: None,
    }
}

#[must_use]
pub fn rust_opaque_handle_type(target: &str) -> String {
    format!(
        "::sifr_runtime::interop::Handle<{}>",
        absolute_runtime_target(target)
    )
}

#[must_use]
pub fn is_rust_generated_bridge_type_path(rust_type_path: &str) -> bool {
    is_generated_bridge_type_path(rust_type_path)
}

pub(crate) fn unsupported_type(ty: &Type, reason: &str) -> RustBridgeTypeContract {
    RustBridgeTypeContract {
        sifr_type: ty.display_name(),
        rust_borrowed_type: None,
        rust_owned_type: None,
        rust_return_type: None,
        kind: RustBridgeTypeKind::Unsupported,
        unsupported_reason: Some(reason.to_string()),
    }
}

fn bridge_param_convention(convention: ParamConvention) -> RustBridgeParamConvention {
    match (convention.ownership(), convention.mutability()) {
        (ParamOwnership::Borrow, sifr_type_system::ParamMutability::Mutable) => {
            RustBridgeParamConvention::MutableBorrow
        }
        (ParamOwnership::Borrow, sifr_type_system::ParamMutability::Immutable) => {
            RustBridgeParamConvention::Borrow
        }
        (ParamOwnership::Own, sifr_type_system::ParamMutability::Immutable) => {
            RustBridgeParamConvention::Own
        }
        (ParamOwnership::Own, sifr_type_system::ParamMutability::Mutable) => {
            RustBridgeParamConvention::OwnMutable
        }
    }
}
