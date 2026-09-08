use super::rust_interop_probe_policy::DirectProbePolicy;
use super::stdlib_interop_demand_tests::{generated, owners, resolve};
use sifr_codegen::{RustInteropOwner, observe_stdlib_interop_selection};
use sifr_ir::{HirExpr, HirModule, HirParam, HirStmt};
use sifr_type_system::{ParamConvention, Type};
use std::collections::BTreeMap;
use std::sync::Arc;

fn lowered(source: &str, defs: &sifr_lowering::ExternalDefs) -> HirModule {
    let parsed = sifr_syntax::parse_module_raw(source, None).unwrap();
    assert!(parsed.has_valid_syntax(), "{:?}", parsed.errors());
    sifr_lowering::lower_module_with_externals(parsed.suite(), defs)
        .unwrap()
        .module
}

#[test]
fn stdlib_interop_startup_project_selection_is_once_per_complete_application() {
    let stdlib = crate::stdlib::compile_stdlib().unwrap();
    let first = lowered("def main():\n    pass\n", &stdlib.defs);
    let last = lowered(
        "from sifr.calendar import isleap\ndef last() -> bool:\n    return isleap(2024)\n",
        &stdlib.defs,
    );
    let (single, count) = observe_stdlib_interop_selection(|| {
        sifr_codegen::generate_rust_with_stdlib(&last, &stdlib.code)
    });
    assert_eq!(count, 1);
    let expected = owners(&single.interop.stdlib_demand);
    assert!(
        expected
            .iter()
            .any(|name| name.starts_with("_sifr.calendar."))
    );
    for modules in [
        vec![("main", &first), ("last", &last)],
        vec![("last", &last), ("main", &first)],
    ] {
        let (project, count) = observe_stdlib_interop_selection(|| {
            sifr_codegen::generate_rust_multi_with_metadata(&modules, &stdlib.code)
        });
        assert_eq!(count, 1);
        assert_eq!(owners(&project.interop.stdlib_demand), expected);
        assert!(!project.rust_files["last"].is_empty());
    }
    // Exercise the driver used by sifr test, not only a synthetic codegen call.
    let dir = std::env::temp_dir().join(format!("b46-test-project-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("helper.sifr"),
        "def value() -> int:\n    return 1\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("test_a.sifr"),
        "from helper import value\ndef test_first():\n    assert value() == 1\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("test_z.sifr"),
        "from sifr.calendar import isleap\ndef test_last():\n    assert isleap(2024)\n",
    )
    .unwrap();
    let paths = BTreeMap::from([
        ("test_a".into(), dir.join("test_a.sifr")),
        ("test_z".into(), dir.join("test_z.sifr")),
    ]);
    let (project, count) = observe_stdlib_interop_selection(|| {
        crate::test_runner::build_test_runner_project(
            &dir,
            &paths,
            &mut sifr_frontend::DiskSourceProvider::new(),
        )
    });
    let project = project.unwrap();
    assert_eq!(count, 1);
    assert_eq!(owners(&project.interop.stdlib_demand), expected);
    assert!(project.all_rust_code.contains("test_last"));
    assert!(project.support_rust_files.contains_key("helper"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn stdlib_interop_startup_readonly_walk_preserves_hidden_edges() {
    let mut stdlib = crate::stdlib::compile_stdlib().unwrap();
    let private = "@rust.opaque(type=sifr_stdlib.Resource, close=close, send=False, sync=False)\nclass Hidden:\n    @rust(Self.close, panic=trusted_no_panic)\n    def close(own self) -> Result[None, ValueError]: ...\n\n@rust(sifr_stdlib.make)\ndef default_value() -> int: ...\n";
    for name in ["_sifr.startup_a", "_sifr.startup_b"] {
        let parsed = sifr_syntax::parse_module_raw(private, None).unwrap();
        let module = sifr_lowering::lower_module_sysroot_private_declaration_with_externals(
            parsed.suite(),
            &stdlib.defs,
        )
        .unwrap();
        sifr_frontend::collect_module_exports(name, &module, &mut stdlib.defs);
        Arc::make_mut(&mut stdlib.code.hir_modules).insert(name.into(), Arc::new(module.module));
    }
    let mut wrapper = lowered("def exposed():\n    pass\n", &stdlib.defs);
    let mut nested = wrapper.functions[0].clone();
    nested.name = "nested".into();
    nested.params.push(HirParam {
        name: "value".into(),
        ty: Type::Int,
        default: Some(HirExpr::Call {
            func: "_sifr.startup_b.default_value".into(),
            args: vec![],
            mutable_arg_places: vec![],
            ty: Type::Int,
        }),
        keyword_only: false,
        convention: ParamConvention::borrow(),
    });
    let hidden_type = |name: &str| Type::Class {
        identity: Some(format!("{name}.Hidden")),
        name: "Hidden".into(),
        type_args: vec![],
        fields: vec![],
        methods: vec![],
        parent_class: None,
    };
    // An empty handler has no expression carrying these types: they exist only
    // in statement metadata. Nested signatures and defaults are separate edges.
    nested.return_type = hidden_type("_sifr.startup_b");
    wrapper.functions[0].body = vec![
        HirStmt::TryExcept {
            body: vec![],
            handlers: vec![],
            body_error_types: vec![hidden_type("_sifr.startup_a")],
        },
        HirStmt::NestedFunction {
            func: nested,
            move_captures: false,
            capture_clones: vec![],
        },
    ];
    let original = format!("{wrapper:?}");
    let mut types = Vec::new();
    let mut functions = Vec::new();
    let mut exprs = std::collections::HashSet::new();
    sifr_ir::visit_hir_function(&wrapper.functions[0], &mut |node| match node {
        sifr_ir::HirNode::Type(Type::Class {
            identity: Some(id), ..
        }) => types.push(id.clone()),
        sifr_ir::HirNode::Expr(expr) => {
            assert!(exprs.insert(std::ptr::from_ref(expr)));
        }
        sifr_ir::HirNode::Function(function) => functions.push(function.name.clone()),
        _ => {}
    });
    assert_eq!(types, ["_sifr.startup_a.Hidden", "_sifr.startup_b.Hidden"]);
    assert_eq!(functions, ["nested", "exposed"]);
    assert_eq!(exprs.len(), 1);
    assert_eq!(format!("{wrapper:?}"), original);
    Arc::make_mut(&mut stdlib.code.hir_modules).insert("sifr.startup".into(), Arc::new(wrapper));
    let mut app = lowered("def main():\n    pass\n", &stdlib.defs);
    app.imports.push(sifr_ir::HirImport {
        module: "sifr.startup".into(),
        names: vec!["exposed".into()],
        aliases: vec![],
    });
    let output = sifr_codegen::generate_rust_multi_with_metadata(&[("main", &app)], &stdlib.code);
    let selected = owners(&output.interop.stdlib_demand);
    for name in [
        "_sifr.startup_a.Hidden",
        "_sifr.startup_a.Hidden.close",
        "_sifr.startup_b.Hidden",
        "_sifr.startup_b.Hidden.close",
        "_sifr.startup_b.default_value",
    ] {
        assert!(selected.contains(name), "{selected:?}");
    }
    assert!(!selected.contains("_sifr.startup_a.default_value"));
    let (open, _) = generated("def main():\n    _ = open(\"missing\", \"rb\")\n");
    let selected = owners(&open.interop.stdlib_demand);
    assert!(selected.contains("_sifr.fs._open_file"));
    assert!(selected.contains("_sifr.fs._file_close"));
    let (mut python, stdlib) = generated(
        "from sifr.python import from_bool, Object, PythonError\ndef main():\n    try:\n        value: Object = from_bool(True)\n        _ = value.get_attr(\"attribute\")\n    except PythonError:\n        pass\n",
    );
    assert!(owners(&python.interop.stdlib_demand).contains("_sifr.python.py_get_attr"));
    assert!(
        python
            .interop
            .stdlib_demand
            .bridge_contracts
            .generated_types
            .iter()
            .any(|t| t.name == "PythonErrorBridge" && t.fields.len() == 5)
    );
    let signature = python
        .interop
        .stdlib_demand
        .bridge_contracts
        .signatures
        .iter_mut()
        .find(|s| matches!(&s.owner, RustInteropOwner::Function { name } if name == "py_from_bool"))
        .unwrap();
    signature.params[0].ty.rust_owned_type = Some("String".into());
    signature.params[0].ty.rust_borrowed_type = Some("&str".into());
    let errors = resolve(python, &stdlib, DirectProbePolicy::ExecuteAll).unwrap_err();
    assert!(errors.iter().any(|e| e.code == "SIFR-RUST-TYPE-0001"));
}
