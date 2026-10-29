use super::*;
use sifr_ir::{HirExpr, HirFunction, HirStmt, MethodKind};
use sifr_type_system::{StructuralRecordType, Type};

fn record_module(name: &str, fields: Vec<(&str, Type)>) -> HirModule {
    let ty = Type::StructuralRecord(StructuralRecordType::new(
        fields
            .into_iter()
            .map(|(field, ty)| (field.to_string(), ty))
            .collect(),
    ));
    HirModule {
        functions: vec![HirFunction {
            name: name.to_string(),
            params: Vec::new(),
            return_type: ty.clone(),
            body: vec![HirStmt::Return {
                value: Some(HirExpr::ConstructorCall {
                    class_name: ty.display_name(),
                    args: vec![
                        HirExpr::StringLiteral("dev@sifr.dev".to_string()),
                        HirExpr::IntLiteral(7),
                    ],
                    ty,
                }),
            }],
            is_async: false,
            method_kind: MethodKind::Regular,
            receiver: None,
            decorators: Vec::new(),
            rust_interop: Vec::new(),
            python_interop: Vec::new(),
            compiler_intrinsic: None,
            type_params: Vec::new(),
        }],
        classes: Vec::new(),
        imports: Vec::new(),
        constants: Vec::new(),
        generic_functions: HashMap::new(),
        type_param_bounds: HashMap::new(),
    }
}

#[test]
fn build_order_and_module_order_reuse_one_crate_root_layout() {
    let first = record_module("first", vec![("id", Type::Int), ("email", Type::Str)]);
    let second = record_module("second", vec![("email", Type::Str), ("id", Type::Int)]);
    let generated = crate::generate_rust_multi_with_metadata(
        &[("second", &second), ("main", &first)],
        &StdlibCode::default(),
    )
    .expect("project generation should succeed");

    assert_eq!(
        generated
            .project_union_prelude
            .matches("pub struct __SifrRecord_")
            .count(),
        1,
        "{}",
        generated.project_union_prelude
    );
    assert!(
        generated
            .rust_files
            .values()
            .all(|source| !source.contains("pub struct __SifrRecord_"))
    );
}

#[test]
fn crate_root_body_does_not_import_its_local_structural_layout() {
    let main = record_module("main_record", vec![("value", Type::Int)]);
    let generated =
        crate::generate_rust_multi_with_metadata(&[("main", &main)], &StdlibCode::default())
            .expect("project generation should succeed");
    let main_rust = &generated.rust_files["main"];

    assert!(
        generated
            .project_union_prelude
            .contains("pub struct __SifrRecord_")
    );
    assert!(
        !main_rust.contains("use crate::__SifrRecord_"),
        "{main_rust}"
    );
}

#[test]
fn support_module_imports_its_crate_root_structural_layout() {
    let main = record_module("main_record", vec![("value", Type::Int)]);
    let support = record_module("support_record", vec![("value", Type::Int)]);
    let generated = crate::generate_rust_multi_with_metadata(
        &[("main", &main), ("support", &support)],
        &StdlibCode::default(),
    )
    .expect("project generation should succeed");
    let support_rust = &generated.rust_files["support"];

    assert!(
        support_rust.contains("use crate::__SifrRecord_"),
        "{support_rust}"
    );
}
