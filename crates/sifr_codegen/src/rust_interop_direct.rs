use sifr_ir::{HirFunction, RustInteropDeclaration, RustInteropDecoratorKind, RustTargetPath};

use crate::{RustExpr, RustStmt};

const BRIDGE_ROOT: &str = "bridge";
const SELF_ROOT: &str = "Self";

pub(crate) fn direct_rust_function_body(func: &HirFunction) -> Option<Vec<RustStmt>> {
    let declaration = direct_rust_function_declaration(func)?;
    let target = declaration.target.as_ref()?;
    let call = RustExpr::FnCall {
        func: Box::new(RustExpr::Path(target.segments.clone())),
        args: func
            .params
            .iter()
            .map(|param| RustExpr::Ident(param.name.clone()))
            .collect(),
    };
    let value = if direct_rust_function_is_async(func, declaration) {
        RustExpr::Await(Box::new(call))
    } else {
        call
    };
    if func.return_type == sifr_type_system::Type::None {
        Some(vec![RustStmt::Expr(value)])
    } else {
        Some(vec![RustStmt::Return(Some(value))])
    }
}

fn direct_rust_function_declaration(func: &HirFunction) -> Option<&RustInteropDeclaration> {
    func.rust_interop
        .iter()
        .find(|declaration| is_direct_function_declaration(declaration))
}

fn is_direct_function_declaration(declaration: &RustInteropDeclaration) -> bool {
    matches!(
        declaration.kind,
        RustInteropDecoratorKind::Function | RustInteropDecoratorKind::Async
    ) && declaration
        .target
        .as_ref()
        .is_some_and(is_direct_cargo_target)
}

fn is_direct_cargo_target(target: &RustTargetPath) -> bool {
    target
        .segments
        .first()
        .is_some_and(|root| root != BRIDGE_ROOT && root != SELF_ROOT)
}

fn direct_rust_function_is_async(func: &HirFunction, declaration: &RustInteropDeclaration) -> bool {
    func.is_async
        || declaration.kind == RustInteropDecoratorKind::Async
        || declaration.abi_requirements.async_boundary
}

#[cfg(test)]
mod tests {
    use ruff_text_size::TextRange;
    use sifr_ir::{
        HirParam, MethodKind, RustInteropAbiRequirements, RustInteropEffect, RustTargetPath,
    };
    use sifr_type_system::{FixedIntType, ParamConvention, Type};

    use super::*;

    #[test]
    fn direct_rust_function_body_calls_cargo_dependency_path() {
        let func = HirFunction {
            name: "crc32".to_string(),
            params: vec![HirParam {
                name: "data".to_string(),
                ty: Type::Bytes,
                default: None,
                keyword_only: false,
                convention: ParamConvention::borrow(),
            }],
            return_type: Type::FixedInt(FixedIntType::U32),
            body: Vec::new(),
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: Vec::new(),
            rust_interop: vec![declaration(
                RustInteropDecoratorKind::Function,
                &["crc32fast", "hash"],
            )],
            type_params: Vec::new(),
        };

        assert_eq!(
            direct_rust_function_body(&func),
            Some(vec![RustStmt::Return(Some(RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    "crc32fast".to_string(),
                    "hash".to_string()
                ])),
                args: vec![RustExpr::Ident("data".to_string())],
            }))])
        );
    }

    #[test]
    fn direct_rust_function_body_skips_reserved_bridge_roots() {
        let mut func = HirFunction {
            name: "digest".to_string(),
            params: Vec::new(),
            return_type: Type::None,
            body: Vec::new(),
            is_async: false,
            method_kind: MethodKind::Regular,
            decorators: Vec::new(),
            rust_interop: vec![declaration(
                RustInteropDecoratorKind::Function,
                &["bridge", "hash", "digest"],
            )],
            type_params: Vec::new(),
        };
        assert_eq!(direct_rust_function_body(&func), None);

        func.rust_interop[0] = declaration(RustInteropDecoratorKind::Function, &["Self", "poll"]);
        assert_eq!(direct_rust_function_body(&func), None);
    }

    #[test]
    fn direct_rust_function_body_awaits_async_targets() {
        let func = HirFunction {
            name: "fetch".to_string(),
            params: Vec::new(),
            return_type: Type::Bool,
            body: Vec::new(),
            is_async: true,
            method_kind: MethodKind::Regular,
            decorators: Vec::new(),
            rust_interop: vec![declaration(
                RustInteropDecoratorKind::Async,
                &["remote", "fetch_ready"],
            )],
            type_params: Vec::new(),
        };

        assert_eq!(
            direct_rust_function_body(&func),
            Some(vec![RustStmt::Return(Some(RustExpr::Await(Box::new(
                RustExpr::FnCall {
                    func: Box::new(RustExpr::Path(vec![
                        "remote".to_string(),
                        "fetch_ready".to_string()
                    ])),
                    args: Vec::new(),
                }
            ))))])
        );
    }

    fn declaration(kind: RustInteropDecoratorKind, segments: &[&str]) -> RustInteropDeclaration {
        RustInteropDeclaration {
            kind,
            target: Some(RustTargetPath {
                segments: segments
                    .iter()
                    .map(|segment| (*segment).to_string())
                    .collect(),
                span: TextRange::default(),
            }),
            arguments: Vec::new(),
            span: TextRange::default(),
            effect: RustInteropEffect::Sync,
            abi_requirements: RustInteropAbiRequirements::default(),
        }
    }
}
