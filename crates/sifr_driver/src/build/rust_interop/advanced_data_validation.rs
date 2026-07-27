use super::{canonical_sifr_target_path, RustInteropResolver};
use sifr_codegen::{RustBridgeParamConvention, RustBridgeSignatureContract};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{RustInteropDecoratorKind, RustInteropValue};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AdvancedDataKind {
    ArrowArray,
    ArrowRecordBatch,
    DataFrame,
    Tensor,
    Dlpack,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AdvancedDataOwnership {
    Borrowed,
    Owned,
    Transfer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AdvancedDataContract {
    kind: AdvancedDataKind,
    owner: String,
    ownership: AdvancedDataOwnership,
    schema: Option<String>,
    dtype: Option<String>,
    rank: Option<i64>,
    shape: Option<Vec<i64>>,
    layout: Option<String>,
    strides: Option<Vec<i64>>,
    device: Option<String>,
    protocol: Option<String>,
}

impl RustInteropResolver<'_> {
    pub(super) fn validate_advanced_data_contracts(
        &mut self,
        declarations: &[sifr_codegen::RustInteropPlanDeclaration],
    ) {
        let mut by_target: BTreeMap<String, Vec<&sifr_codegen::RustInteropPlanDeclaration>> =
            BTreeMap::new();
        for declaration in declarations {
            if matches!(
                declaration.declaration.kind,
                RustInteropDecoratorKind::Function | RustInteropDecoratorKind::View
            ) {
                by_target
                    .entry(canonical_sifr_target_path(declaration))
                    .or_default()
                    .push(declaration);
            }
        }

        for declarations in by_target.values() {
            self.validate_advanced_data_group(declarations);
        }
    }

    fn validate_advanced_data_group(
        &mut self,
        declarations: &[&sifr_codegen::RustInteropPlanDeclaration],
    ) {
        let Some(view_declaration) = declarations
            .iter()
            .find(|declaration| declaration.declaration.kind == RustInteropDecoratorKind::View)
        else {
            return;
        };
        let Some(contract) =
            parse_advanced_data_contract(view_declaration).unwrap_or_else(|reason| {
                self.push_advanced_data_diagnostic(view_declaration, reason);
                None
            })
        else {
            return;
        };

        self.validate_shared_bridge_root(declarations, view_declaration, contract.kind);
        validate_contract_shape(&contract).unwrap_or_else(|reason| {
            self.push_advanced_data_diagnostic(view_declaration, reason);
        });
        self.validate_signature_ownership(declarations, view_declaration, &contract);
    }

    fn validate_shared_bridge_root(
        &mut self,
        declarations: &[&sifr_codegen::RustInteropPlanDeclaration],
        view_declaration: &sifr_codegen::RustInteropPlanDeclaration,
        kind: AdvancedDataKind,
    ) {
        let Some(root) = declarations.iter().find_map(|declaration| {
            if declaration.declaration.kind != RustInteropDecoratorKind::Function {
                return None;
            }
            declaration
                .declaration
                .target
                .as_ref()
                .and_then(|target| target.segments.first())
                .map(String::as_str)
        }) else {
            return;
        };
        let expected = match kind {
            AdvancedDataKind::ArrowArray
            | AdvancedDataKind::ArrowRecordBatch
            | AdvancedDataKind::DataFrame => "sifr_arrow_bridge",
            AdvancedDataKind::Tensor | AdvancedDataKind::Dlpack => "sifr_tensor_bridge",
        };
        if root != expected {
            self.push_advanced_data_diagnostic(
                view_declaration,
                "advanced data views must target the matching shared bridge crate",
            );
        }
    }

    fn validate_signature_ownership(
        &mut self,
        declarations: &[&sifr_codegen::RustInteropPlanDeclaration],
        view_declaration: &sifr_codegen::RustInteropPlanDeclaration,
        contract: &AdvancedDataContract,
    ) {
        if contract.kind != AdvancedDataKind::Dlpack
            || contract.ownership != AdvancedDataOwnership::Transfer
        {
            return;
        }
        let key = canonical_sifr_target_path(declarations[0]);
        let Some(signature) = self.signature_contracts.get(&key).cloned() else {
            return;
        };
        if !owner_param_is_owned(&signature, &contract.owner) {
            self.push_advanced_data_diagnostic(
                view_declaration,
                "DLPack transfer ownership requires an owned owner parameter",
            );
        }
    }

    fn push_advanced_data_diagnostic(
        &mut self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
        reason: &'static str,
    ) {
        self.push_diagnostic(
            declaration,
            declaration.declaration.span,
            DiagnosticCode::RUST_ZERO_COPY_CONTRACT,
            "invalid Rust advanced data/view contract: {reason}",
            vec![("reason", reason.to_string())],
            Vec::new(),
            None,
        );
    }
}

pub(super) fn is_advanced_view_key(name: &str) -> bool {
    matches!(
        name,
        "data"
            | "schema"
            | "dtype"
            | "rank"
            | "shape"
            | "layout"
            | "strides"
            | "device"
            | "ownership"
            | "protocol"
    )
}

fn parse_advanced_data_contract(
    declaration: &sifr_codegen::RustInteropPlanDeclaration,
) -> Result<Option<AdvancedDataContract>, &'static str> {
    let mut saw_advanced_key = false;
    let mut kind = None;
    let mut owner = None;
    let mut ownership = None;
    let mut schema = None;
    let mut dtype = None;
    let mut rank = None;
    let mut shape = None;
    let mut layout = None;
    let mut strides = None;
    let mut device = None;
    let mut protocol = None;

    for argument in &declaration.declaration.arguments {
        let Some(name) = argument.name.as_deref() else {
            continue;
        };
        if name == "owner" {
            if let RustInteropValue::Symbol(symbol) = &argument.value {
                owner = Some(symbol.clone());
            }
            continue;
        }
        if !is_advanced_view_key(name) {
            continue;
        }
        saw_advanced_key = true;
        match name {
            "data" => kind = Some(advanced_data_kind(&argument.value)?),
            "ownership" => ownership = Some(advanced_data_ownership(&argument.value)?),
            "schema" => match &argument.value {
                RustInteropValue::TargetPath(path)
                    if path_root_is(&path.dotted(), "sifr_arrow_bridge") =>
                {
                    schema = Some(path.dotted());
                }
                RustInteropValue::TargetPath(_) => {
                    return Err("`schema=` must be a dotted `sifr_arrow_bridge` schema path");
                }
                _ => return Err("`schema=` must be a dotted `sifr_arrow_bridge` schema path"),
            },
            "dtype" => dtype = Some(dtype_name(&argument.value)?.to_string()),
            "rank" => match &argument.value {
                RustInteropValue::Integer(value) if *value >= 0 => rank = Some(*value),
                _ => return Err("`rank=` must be a non-negative integer"),
            },
            "shape" => shape = Some(shape_value(&argument.value)?),
            "layout" => layout = Some(layout_name(&argument.value)?.to_string()),
            "strides" => strides = Some(strides_value(&argument.value)?),
            "device" => device = Some(device_name(&argument.value)?.to_string()),
            "protocol" => match &argument.value {
                RustInteropValue::TargetPath(path)
                    if path_root_is(&path.dotted(), "sifr_tensor_bridge") =>
                {
                    protocol = Some(path.dotted());
                }
                RustInteropValue::TargetPath(_) => {
                    return Err("`protocol=` must be a dotted `sifr_tensor_bridge` protocol path");
                }
                _ => return Err("`protocol=` must be a dotted `sifr_tensor_bridge` protocol path"),
            },
            _ => {}
        }
    }

    if !saw_advanced_key {
        return Ok(None);
    }
    Ok(Some(AdvancedDataContract {
        kind: kind.ok_or("advanced data view metadata requires `data=`")?,
        owner: owner.ok_or("advanced data view metadata requires `owner=`")?,
        ownership: ownership.ok_or("advanced data view metadata requires `ownership=`")?,
        schema,
        dtype,
        rank,
        shape,
        layout,
        strides,
        device,
        protocol,
    }))
}

fn validate_contract_shape(contract: &AdvancedDataContract) -> Result<(), &'static str> {
    match contract.kind {
        AdvancedDataKind::ArrowArray
        | AdvancedDataKind::ArrowRecordBatch
        | AdvancedDataKind::DataFrame => {
            if contract.schema.is_none() {
                return Err("Arrow and dataframe views require `schema=`");
            }
            if contract.dtype.is_some()
                || contract.rank.is_some()
                || contract.shape.is_some()
                || contract.layout.is_some()
                || contract.strides.is_some()
                || contract.device.is_some()
                || contract.protocol.is_some()
            {
                return Err("Arrow and dataframe views cannot declare tensor metadata keys");
            }
            if contract.ownership == AdvancedDataOwnership::Transfer {
                return Err(
                    "Arrow and dataframe views use `ownership=borrowed` or `ownership=owned`",
                );
            }
        }
        AdvancedDataKind::Tensor | AdvancedDataKind::Dlpack => {
            if contract.dtype.is_none()
                || contract.shape.is_none()
                || contract.layout.is_none()
                || contract.strides.is_none()
                || contract.device.is_none()
            {
                return Err(
                    "tensor and DLPack views require `dtype=`, `shape=`, `layout=`, `strides=`, and `device=`",
                );
            }
            if contract.schema.is_some() {
                return Err("tensor and DLPack views cannot declare `schema=`");
            }
            validate_tensor_rank_shape_strides(contract)?;
            if contract.kind == AdvancedDataKind::Dlpack {
                if contract.ownership != AdvancedDataOwnership::Transfer {
                    return Err("DLPack handoff requires `ownership=transfer`");
                }
                if contract.protocol.is_none() {
                    return Err("DLPack handoff requires `protocol=`");
                }
            } else if contract.protocol.is_some() {
                return Err("`protocol=` is only supported for DLPack handoff");
            }
        }
    }
    Ok(())
}

fn validate_tensor_rank_shape_strides(contract: &AdvancedDataContract) -> Result<(), &'static str> {
    let Some(shape) = contract.shape.as_ref() else {
        return Ok(());
    };
    let Some(strides) = contract.strides.as_ref() else {
        return Ok(());
    };
    if shape.len() != strides.len() {
        return Err("tensor `shape=` and `strides=` must have the same rank");
    }
    if let Some(rank) = contract.rank {
        if rank != i64::try_from(shape.len()).unwrap_or(i64::MAX) {
            return Err("tensor `rank=` must match `shape=` and `strides=` length");
        }
    }
    Ok(())
}

fn advanced_data_kind(value: &RustInteropValue) -> Result<AdvancedDataKind, &'static str> {
    match value {
        RustInteropValue::Symbol(symbol) if symbol == "arrow_array" => {
            Ok(AdvancedDataKind::ArrowArray)
        }
        RustInteropValue::Symbol(symbol) if symbol == "arrow_record_batch" => {
            Ok(AdvancedDataKind::ArrowRecordBatch)
        }
        RustInteropValue::Symbol(symbol) if symbol == "dataframe" => {
            Ok(AdvancedDataKind::DataFrame)
        }
        RustInteropValue::Symbol(symbol) if symbol == "tensor" => Ok(AdvancedDataKind::Tensor),
        RustInteropValue::Symbol(symbol) if symbol == "dlpack" => Ok(AdvancedDataKind::Dlpack),
        _ => Err("`data=` must be arrow_array, arrow_record_batch, dataframe, tensor, or dlpack"),
    }
}

fn advanced_data_ownership(
    value: &RustInteropValue,
) -> Result<AdvancedDataOwnership, &'static str> {
    match value {
        RustInteropValue::Symbol(symbol) if symbol == "borrowed" => {
            Ok(AdvancedDataOwnership::Borrowed)
        }
        RustInteropValue::Symbol(symbol) if symbol == "owned" => Ok(AdvancedDataOwnership::Owned),
        RustInteropValue::Symbol(symbol) if symbol == "transfer" => {
            Ok(AdvancedDataOwnership::Transfer)
        }
        _ => Err("`ownership=` must be borrowed, owned, or transfer"),
    }
}

fn dtype_name(value: &RustInteropValue) -> Result<&str, &'static str> {
    let RustInteropValue::Symbol(symbol) = value else {
        return Err("`dtype=` must be a supported tensor dtype symbol");
    };
    match symbol.as_str() {
        "bool" | "i8" | "u8" | "i16" | "u16" | "i32" | "u32" | "i64" | "u64" | "f16" | "f32"
        | "bf16" | "f64" => Ok(symbol),
        _ => Err("`dtype=` must be a supported tensor dtype symbol"),
    }
}

fn shape_value(value: &RustInteropValue) -> Result<Vec<i64>, &'static str> {
    let RustInteropValue::IntegerList(values) = value else {
        return Err("`shape=` must be a non-empty integer list");
    };
    if values.iter().any(|value| *value < 0) {
        return Err("`shape=` dimensions must be non-negative integers");
    }
    Ok(values.clone())
}

fn strides_value(value: &RustInteropValue) -> Result<Vec<i64>, &'static str> {
    let RustInteropValue::IntegerList(values) = value else {
        return Err("`strides=` must be a non-empty integer list");
    };
    Ok(values.clone())
}

fn layout_name(value: &RustInteropValue) -> Result<&str, &'static str> {
    let RustInteropValue::Symbol(symbol) = value else {
        return Err("`layout=` must be contiguous or strided");
    };
    match symbol.as_str() {
        "contiguous" | "strided" => Ok(symbol),
        _ => Err("`layout=` must be contiguous or strided"),
    }
}

fn device_name(value: &RustInteropValue) -> Result<&str, &'static str> {
    let RustInteropValue::Symbol(symbol) = value else {
        return Err("`device=` must be cpu");
    };
    match symbol.as_str() {
        "cpu" => Ok(symbol),
        _ => Err("`device=` must be cpu"),
    }
}

fn owner_param_is_owned(signature: &RustBridgeSignatureContract, owner: &str) -> bool {
    signature.params.iter().any(|param| {
        param.name == owner
            && matches!(
                param.convention,
                RustBridgeParamConvention::Own | RustBridgeParamConvention::OwnMutable
            )
    })
}

fn path_root_is(path: &str, expected_root: &str) -> bool {
    path.split('.').next() == Some(expected_root)
}
