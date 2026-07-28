use super::{canonical_sifr_target_path, RustInteropResolver};
use sifr_codegen::{RustBridgeParamConvention, RustBridgeSignatureContract};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{RustInteropDecoratorKind, RustInteropValue, RustTargetPath};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq)]
struct ZeroCopyContract {
    owner: String,
    view: RustTargetPath,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ViewLifetime {
    Call,
    Owner,
    Static,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ViewMutability {
    Immutable,
    Mutable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ViewContract {
    owner: String,
    lifetime: ViewLifetime,
    mutability: ViewMutability,
    send: bool,
    sync: bool,
}

impl RustInteropResolver<'_> {
    pub(super) fn validate_zero_copy_contracts(
        &mut self,
        declarations: &[sifr_codegen::RustInteropPlanDeclaration],
    ) {
        let mut by_target: BTreeMap<String, Vec<&sifr_codegen::RustInteropPlanDeclaration>> =
            BTreeMap::new();
        for declaration in declarations {
            if matches!(
                declaration.declaration.kind,
                RustInteropDecoratorKind::ZeroCopy | RustInteropDecoratorKind::View
            ) {
                by_target
                    .entry(canonical_sifr_target_path(declaration))
                    .or_default()
                    .push(declaration);
            }
        }

        for declarations in by_target.values() {
            self.validate_zero_copy_group(declarations);
        }
    }

    fn validate_zero_copy_group(
        &mut self,
        declarations: &[&sifr_codegen::RustInteropPlanDeclaration],
    ) {
        let mut zero_copy = None;
        let mut view = None;
        let mut saw_view_declaration = false;

        for declaration in declarations {
            match declaration.declaration.kind {
                RustInteropDecoratorKind::ZeroCopy => {
                    if zero_copy.is_some() {
                        self.push_zero_copy_diagnostic(
                            declaration,
                            "duplicate `@rust.zero_copy(...)` declaration",
                        );
                        continue;
                    }
                    zero_copy = parse_zero_copy_contract(declaration).map_or_else(
                        |reason| {
                            self.push_zero_copy_diagnostic(declaration, reason);
                            None
                        },
                        Some,
                    );
                }
                RustInteropDecoratorKind::View => {
                    saw_view_declaration = true;
                    if view.is_some() {
                        self.push_zero_copy_diagnostic(
                            declaration,
                            "duplicate `@rust.view(...)` declaration",
                        );
                        continue;
                    }
                    view = parse_view_contract(declaration).map_or_else(
                        |reason| {
                            self.push_zero_copy_diagnostic(declaration, reason);
                            None
                        },
                        Some,
                    );
                }
                RustInteropDecoratorKind::Function
                | RustInteropDecoratorKind::Opaque
                | RustInteropDecoratorKind::Async
                | RustInteropDecoratorKind::Callback => {}
            }
        }

        let Some(view) = view else {
            if !saw_view_declaration {
                if let Some(zero_copy_declaration) = declarations.iter().find(|declaration| {
                    declaration.declaration.kind == RustInteropDecoratorKind::ZeroCopy
                }) {
                    self.push_zero_copy_diagnostic(
                        zero_copy_declaration,
                        "`@rust.zero_copy(...)` requires a paired `@rust.view(...)` declaration",
                    );
                }
            }
            return;
        };
        self.zero_copy_probe_obligations.insert(
            canonical_sifr_target_path(declarations[0]),
            (view.send, view.sync),
        );

        if view.lifetime == ViewLifetime::Call {
            let Some(view_declaration) = declarations
                .iter()
                .find(|declaration| declaration.declaration.kind == RustInteropDecoratorKind::View)
            else {
                return;
            };
            self.push_zero_copy_diagnostic(
                view_declaration,
                "returned Rust views cannot declare `lifetime=call`",
            );
        }
        if view.lifetime != ViewLifetime::Static
            && declarations
                .iter()
                .any(|declaration| declaration.declaration.abi_requirements.async_boundary)
        {
            let Some(view_declaration) = declarations
                .iter()
                .find(|declaration| declaration.declaration.kind == RustInteropDecoratorKind::View)
            else {
                return;
            };
            self.push_zero_copy_diagnostic(
                view_declaration,
                "async Rust interop views must use `lifetime=static` until borrowed view suspension is supported",
            );
        }

        if let Some(zero_copy) = &zero_copy {
            if zero_copy.owner != view.owner {
                let Some(zero_copy_declaration) = declarations.iter().find(|declaration| {
                    declaration.declaration.kind == RustInteropDecoratorKind::ZeroCopy
                }) else {
                    return;
                };
                self.push_zero_copy_diagnostic(
                    zero_copy_declaration,
                    "`@rust.zero_copy(...)` and `@rust.view(...)` must name the same owner",
                );
            }
        }

        let key = canonical_sifr_target_path(declarations[0]);
        let Some(signature) = self.signature_contracts.get(&key).cloned() else {
            return;
        };
        if signature_has_unsupported_type(&signature) {
            return;
        }
        if let Some(zero_copy) = zero_copy {
            self.validate_view_return(&signature, declarations, &zero_copy.view);
        }
        self.validate_view_owner(&signature, declarations, &view);
    }

    fn validate_view_owner(
        &mut self,
        signature: &RustBridgeSignatureContract,
        declarations: &[&sifr_codegen::RustInteropPlanDeclaration],
        view: &ViewContract,
    ) {
        let Some(view_declaration) = declarations
            .iter()
            .find(|declaration| declaration.declaration.kind == RustInteropDecoratorKind::View)
        else {
            return;
        };
        let Some(owner_param) = signature
            .params
            .iter()
            .find(|param| param.name == view.owner)
        else {
            self.push_zero_copy_diagnostic(
                view_declaration,
                "`@rust.view(...)` owner must name a Sifr parameter",
            );
            return;
        };
        if view.mutability == ViewMutability::Mutable
            && owner_param.convention == RustBridgeParamConvention::Borrow
        {
            self.push_zero_copy_diagnostic(
                view_declaration,
                "mutable Rust views require an exclusive owner parameter",
            );
        }
    }

    fn validate_view_return(
        &mut self,
        signature: &RustBridgeSignatureContract,
        declarations: &[&sifr_codegen::RustInteropPlanDeclaration],
        view_type: &RustTargetPath,
    ) {
        let Some(zero_copy_declaration) = declarations
            .iter()
            .find(|declaration| declaration.declaration.kind == RustInteropDecoratorKind::ZeroCopy)
        else {
            return;
        };
        let Some(return_type) = returned_ok_type(signature) else {
            self.push_zero_copy_diagnostic(
                zero_copy_declaration,
                "`view=` requires a concrete Rust view type in the function return value",
            );
            return;
        };
        let rust_view_type = canonical_rust_view_type(view_type);
        let expected = format!("::sifr_runtime::interop::Handle<{rust_view_type}>");
        if return_type != expected {
            self.push_zero_copy_diagnostic(
                zero_copy_declaration,
                "`view=` must name the Rust type carried by the function return value",
            );
        }
    }

    fn push_zero_copy_diagnostic(
        &mut self,
        declaration: &sifr_codegen::RustInteropPlanDeclaration,
        reason: &'static str,
    ) {
        self.push_diagnostic(
            declaration,
            declaration.declaration.span,
            DiagnosticCode::RUST_ZERO_COPY_CONTRACT,
            "invalid Rust zero-copy/view contract: {reason}",
            vec![("reason", reason.to_string())],
            Vec::new(),
            None,
        );
    }
}

fn parse_zero_copy_contract(
    declaration: &sifr_codegen::RustInteropPlanDeclaration,
) -> Result<ZeroCopyContract, &'static str> {
    let mut owner = None;
    let mut view = None;
    for argument in &declaration.declaration.arguments {
        let Some(name) = argument.name.as_deref() else {
            return Err("`@rust.zero_copy(...)` requires named arguments");
        };
        match name {
            "owner" => match &argument.value {
                RustInteropValue::Symbol(symbol) => owner = Some(symbol.clone()),
                _ => return Err("`owner=` must name a Sifr parameter"),
            },
            "view" => match &argument.value {
                RustInteropValue::TargetPath(path) => view = Some(path.clone()),
                _ => return Err("`view=` must be a dotted Rust target path"),
            },
            _ => return Err("unsupported `@rust.zero_copy(...)` key"),
        }
    }
    Ok(ZeroCopyContract {
        owner: owner.ok_or("`@rust.zero_copy(...)` requires `owner=`")?,
        view: view.ok_or("`@rust.zero_copy(...)` requires `view=`")?,
    })
}

fn parse_view_contract(
    declaration: &sifr_codegen::RustInteropPlanDeclaration,
) -> Result<ViewContract, &'static str> {
    let mut owner = None;
    let mut lifetime = None;
    let mut mutability = None;
    let mut send = None;
    let mut sync = None;
    for argument in &declaration.declaration.arguments {
        let Some(name) = argument.name.as_deref() else {
            return Err("`@rust.view(...)` requires named arguments");
        };
        match name {
            "owner" => match &argument.value {
                RustInteropValue::Symbol(symbol) => owner = Some(symbol.clone()),
                _ => return Err("`owner=` must name a Sifr parameter"),
            },
            "lifetime" => lifetime = Some(view_lifetime(&argument.value)?),
            "mutability" => mutability = Some(view_mutability(&argument.value)?),
            "send" => match &argument.value {
                RustInteropValue::Boolean(value) => send = Some(*value),
                _ => return Err("`send=` must be True or False"),
            },
            "sync" => match &argument.value {
                RustInteropValue::Boolean(value) => sync = Some(*value),
                _ => return Err("`sync=` must be True or False"),
            },
            _ if super::advanced_data_validation::is_advanced_view_key(name) => {}
            _ => return Err("unsupported `@rust.view(...)` key"),
        }
    }
    Ok(ViewContract {
        owner: owner.ok_or("`@rust.view(...)` requires `owner=`")?,
        lifetime: lifetime.ok_or("`@rust.view(...)` requires `lifetime=`")?,
        mutability: mutability.ok_or("`@rust.view(...)` requires `mutability=`")?,
        send: send.ok_or("`@rust.view(...)` requires `send=`")?,
        sync: sync.ok_or("`@rust.view(...)` requires `sync=`")?,
    })
}

fn view_lifetime(value: &RustInteropValue) -> Result<ViewLifetime, &'static str> {
    match value {
        RustInteropValue::Symbol(symbol) if symbol == "call" => Ok(ViewLifetime::Call),
        RustInteropValue::Symbol(symbol) if symbol == "owner" => Ok(ViewLifetime::Owner),
        RustInteropValue::Symbol(symbol) if symbol == "static" => Ok(ViewLifetime::Static),
        _ => Err("`lifetime=` must be call, owner, or static"),
    }
}

fn view_mutability(value: &RustInteropValue) -> Result<ViewMutability, &'static str> {
    match value {
        RustInteropValue::Symbol(symbol) if symbol == "immutable" => Ok(ViewMutability::Immutable),
        RustInteropValue::Symbol(symbol) if symbol == "mutable" => Ok(ViewMutability::Mutable),
        _ => Err("`mutability=` must be immutable or mutable"),
    }
}

fn signature_has_unsupported_type(signature: &RustBridgeSignatureContract) -> bool {
    signature.return_type.kind == sifr_codegen::RustBridgeTypeKind::Unsupported
        || signature.return_type.unsupported_reason.is_some()
        || signature.params.iter().any(|param| {
            param.ty.kind == sifr_codegen::RustBridgeTypeKind::Unsupported
                || param.ty.unsupported_reason.is_some()
        })
}

fn returned_ok_type(signature: &RustBridgeSignatureContract) -> Option<&str> {
    let rendered = signature.return_type.rust_return_type.as_deref()?.trim();
    if signature.return_type.kind != sifr_codegen::RustBridgeTypeKind::Result {
        return Some(rendered);
    }
    let inner = rendered
        .strip_prefix("Result<")
        .and_then(|value| value.strip_suffix('>'))?;
    let mut depth = 0_u32;
    for (index, ch) in inner.char_indices() {
        match ch {
            '<' => depth = depth.saturating_add(1),
            '>' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => return Some(inner[..index].trim()),
            _ => {}
        }
    }
    None
}

fn canonical_rust_view_type(view_type: &RustTargetPath) -> String {
    let path = view_type.segments.join("::");
    if matches!(
        view_type.segments.first().map(String::as_str),
        Some("sifr_runtime" | "sifr_stdlib")
    ) {
        format!("::{path}")
    } else {
        path
    }
}
