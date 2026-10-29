    use super::*;
    use sifr_ir::{HirStmt, MethodKind};

    fn stdlib_error(identity: &str) -> Type {
        Type::Class {
            identity: Some(identity.to_string()),
            type_args: Vec::new(),
            name: "Error".to_string(),
            fields: vec![("message".to_string(), Type::Str)].into(),
            methods: Vec::new().into(),
            parent_class: Some("Error".to_string()),
        }
    }

    #[test]
    fn same_basename_stdlib_errors_keep_distinct_nominal_declarations() {
        let mut declarations = BTreeMap::new();
        let mut builtin_types = BTreeMap::new();

        collect_shared_nominals(
            &Type::Union(vec![
                stdlib_error("sifr.csv.Error"),
                stdlib_error("sifr.configparser.Error"),
            ]),
            &mut declarations,
            &mut builtin_types,
        );

        assert_eq!(
            declarations.get("sifr.csv"),
            Some(&HashSet::from(["Error".to_string()]))
        );
        assert_eq!(
            declarations.get("sifr.configparser"),
            Some(&HashSet::from(["Error".to_string()]))
        );
        assert!(builtin_types.is_empty());
    }

    #[test]
    fn same_basename_stdlib_errors_get_distinct_project_paths() {
        let csv = stdlib_error("sifr.csv.Error");
        let config = stdlib_error("sifr.configparser.Error");
        let unions = HashMap::from([("ImportedErrors".to_string(), vec![csv, config])]);

        let plan = project_stdlib_nominal_plan(&unions, &[]);
        let csv_path = plan
            .registry
            .rust_paths
            .get("sifr.csv.Error")
            .expect("csv error should have a canonical project path");
        let config_path = plan
            .registry
            .rust_paths
            .get("sifr.configparser.Error")
            .expect("configparser error should have a canonical project path");

        assert_ne!(csv_path, config_path);
        assert!(csv_path.contains(&class_rust_name(Some("sifr.csv.Error"), "Error")));
        assert!(config_path.contains(&class_rust_name(Some("sifr.configparser.Error"), "Error")));
    }

    #[test]
    fn direct_crate_root_nominal_gets_a_project_path_without_other_nominals() {
        let native_file = Type::Class {
            identity: Some("_sifr.fs.NativeFileHandle".to_string()),
            type_args: Vec::new(),
            name: "NativeFileHandle".to_string(),
            fields: Vec::new().into(),
            methods: Vec::new().into(),
            parent_class: None,
        };
        let unions = HashMap::from([("NativeFile".to_string(), vec![native_file])]);

        let plan = project_stdlib_nominal_plan(&unions, &[]);

        assert_eq!(
            plan.registry.rust_paths.get("_sifr.fs.NativeFileHandle"),
            Some(&format!(
                "crate::{}",
                class_rust_name(Some("_sifr.fs.NativeFileHandle"), "NativeFileHandle")
            ))
        );
    }

    #[test]
    fn builtin_error_union_keeps_its_shared_module_definition() {
        let timeout_error = Type::Class {
            identity: Some("sifr.builtin.TimeoutError".to_string()),
            type_args: Vec::new(),
            name: "TimeoutError".to_string(),
            fields: vec![("message".to_string(), Type::Str)].into(),
            methods: Vec::new().into(),
            parent_class: Some("Error".to_string()),
        };
        let unions = HashMap::from([(
            "TimeoutOrValueError".to_string(),
            vec![
                timeout_error,
                Type::Class {
                    identity: None,
                    type_args: Vec::new(),
                    name: "ValueError".to_string(),
                    fields: vec![("message".to_string(), Type::Str)].into(),
                    methods: Vec::new().into(),
                    parent_class: Some("Error".to_string()),
                },
            ],
        )]);

        let plan = project_stdlib_nominal_plan(&unions, &[]);

        assert!(
            plan.registry
                .rust_paths
                .contains_key("sifr.builtin.TimeoutError")
        );
        assert!(!plan.registry.crate_root_rust_names.contains("TimeoutError"));
    }

    #[test]
    fn task_scope_error_plan_includes_scope_failure_conversion() {
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: Vec::new(),
                return_type: Type::Result(
                    Box::new(Type::None),
                    Box::new(builtin_error_type("Error")),
                ),
                body: vec![HirStmt::AsyncWith {
                    kind: sifr_ir::HirAsyncWithKind::TaskScope,
                    target: Some("scope".to_string()),
                    body: Vec::new(),
                }],
                is_async: true,
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
        };

        let generated =
            crate::generate_rust_multi_with_metadata(&[("main", &module)], &StdlibCode::default())
                .expect("project generation should succeed");
        assert_eq!(
            generated
                .project_union_prelude
                .matches("struct ScopeFailure")
                .count(),
            1,
            "{}",
            generated.project_union_prelude
        );
        assert!(!generated.rust_files["main"].contains("struct ScopeFailure"));
    }

    #[test]
    fn emitted_transitive_nominals_join_the_project_registry() {
        let mut stdlib = StdlibCode::default();
        stdlib.module_rust_code.insert(
            "sifr.io".to_string(),
            crate::StdlibRustSource {
                module: "sifr.io".to_string(),
                source_path: "stdlib/sifr/io.sifr".to_string(),
                source_sha256: "digest".to_string(),
                nominal_types: HashSet::from([
                    "BinaryFileHandle".to_string(),
                    "TextFileHandle".to_string(),
                ]),
                rust: String::new(),
            },
        );
        let shared_source =
            "pub struct __SifrIoBinaryFileHandle;\npub struct __SifrIoTextFileHandle;\n";
        let mut registry = ProjectNominalRegistry::default();
        registry.register_shared(
            "sifr.io.TextFileHandle".to_string(),
            "__SifrIoTextFileHandle".to_string(),
        );

        register_transitive_stdlib_nominals(shared_source, &stdlib, &mut registry);

        assert!(
            registry
                .shared_rust_names
                .contains("__SifrIoBinaryFileHandle")
        );
        assert_eq!(
            registry.rust_paths.get("sifr.io.BinaryFileHandle"),
            Some(&"crate::__sifr_project_nominals::__SifrIoBinaryFileHandle".to_string())
        );
    }

    #[test]
    fn emitted_builtin_nominals_join_the_project_registry() {
        let shared_source =
            "pub struct ParseError;\npub struct FileNotFoundError;\npub struct CustomError;\n";
        let mut registry = ProjectNominalRegistry::default();

        register_emitted_builtin_nominals(shared_source, &mut registry);

        assert!(registry.shared_rust_names.contains("ParseError"));
        assert_eq!(
            registry.rust_paths.get("sifr.builtin.ParseError"),
            Some(&"crate::__sifr_project_nominals::ParseError".to_string())
        );
        assert!(!registry.rust_paths.contains_key("ParseError"));
        assert!(!registry.shared_rust_names.contains("FileNotFoundError"));
        assert!(!registry.rust_paths.contains_key("FileNotFoundError"));
        for name in ["FileNotFoundError", "ValueError", "CustomError"] {
            assert!(!registry.shared_rust_names.contains(name));
            assert!(!registry.rust_paths.contains_key(name));
            assert!(
                !registry
                    .rust_paths
                    .contains_key(&format!("sifr.builtin.{name}"))
            );
        }
        assert_eq!(registry.rust_paths.len(), 1);
    }

    #[test]
    fn builtin_registry_identity_does_not_replace_a_module_qualified_shadow() {
        let mut paths = HashMap::from([
            (
                "shadow.ValueError".to_string(),
                "crate::shadow::ValueError".to_string(),
            ),
            (
                "ValueError".to_string(),
                "crate::shadow::ValueError".to_string(),
            ),
        ]);
        let mut registry = ProjectNominalRegistry::default();
        let builtin = BuiltinError::from_name("ValueError").expect("known builtin fixture");
        registry.register_builtin(builtin, "ValueError".to_string());

        paths.extend(registry.rust_paths);

        assert_eq!(
            paths.get("shadow.ValueError"),
            Some(&"crate::shadow::ValueError".to_string())
        );
        assert_eq!(
            paths.get("ValueError"),
            Some(&"crate::shadow::ValueError".to_string())
        );
        assert_eq!(
            paths.get("sifr.builtin.ValueError"),
            Some(&"crate::__sifr_project_nominals::ValueError".to_string())
        );

        let relocated = relocate_project_stdlib_nominals_owned_by(
            "struct ValueError { message: String, detail: String }\nfn detail(error: ValueError) -> String { error.detail }\n",
            "shadow",
            &HashSet::from(["main"]),
            &HashSet::from(["ValueError".to_string()]),
            &HashSet::from(["ValueError".to_string()]),
            &mut RelocatedStructuralImplementations::default(),
        );
        assert!(relocated.contains("struct ValueError"), "{relocated}");
        assert!(relocated.contains("error.detail"), "{relocated}");
        assert!(!relocated.contains("use crate::ValueError"), "{relocated}");
    }

    #[test]
    fn direct_io_error_kind_type_does_not_create_a_project_nominal() {
        let direct_kind = Type::Class {
            identity: Some("sifr.builtin.FileNotFoundError".to_string()),
            type_args: Vec::new(),
            name: "FileNotFoundError".to_string(),
            fields: vec![
                ("message".to_string(), Type::Str),
                ("kind".to_string(), Type::Str),
            ]
            .into(),
            methods: Vec::new().into(),
            parent_class: Some("IOError".to_string()),
        };
        let unions = HashMap::from([("DirectKind".to_string(), vec![direct_kind])]);

        let plan = project_stdlib_nominal_plan(&unions, &[]);

        assert!(!plan.registry.rust_paths.contains_key("FileNotFoundError"));
        assert!(
            !plan
                .registry
                .rust_paths
                .contains_key("sifr.builtin.FileNotFoundError")
        );
    }

    #[test]
    fn io_error_kind_handlers_do_not_create_dangling_project_nominals() {
        let module = HirModule {
            functions: vec![HirFunction {
                name: "main".to_string(),
                params: Vec::new(),
                return_type: Type::None,
                body: vec![HirStmt::TryExcept {
                    body: vec![HirStmt::Pass],
                    handlers: vec![sifr_ir::HirExceptHandler {
                        error_type: Some("FileNotFoundError".to_string()),
                        error_resolved_type: None,
                        name: None,
                        body: vec![HirStmt::Pass],
                    }],
                    body_error_types: Vec::new(),
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
        };

        let generated =
            crate::generate_rust_multi_with_metadata(&[("main", &module)], &StdlibCode::default())
                .expect("project generation should succeed");

        assert!(
            !generated
                .project_union_prelude
                .contains("FileNotFoundError")
        );
        syn::parse_file(&generated.project_union_prelude)
            .expect("project nominal prelude should not contain a dangling re-export");
    }

    #[test]
    fn relocation_retains_local_child_conversion_into_shared_parent() {
        let mut plan = ProjectStdlibNominalPlan::empty();
        plan.registry.register_shared(
            "sifr.logging.Formatter".to_string(),
            "Formatter".to_string(),
        );
        let source = r#"
struct Formatter { template: String }
struct ChildFormatter { formatter: Formatter }

impl From<ChildFormatter> for Formatter {
    fn from(value: ChildFormatter) -> Self { value.formatter }
}
"#;

        let relocated = relocate_project_stdlib_nominals(
            source,
            "main",
            &plan,
            &HashSet::from(["main"]),
            &HashSet::from(["ChildFormatter".to_string()]),
            &mut RelocatedStructuralImplementations::default(),
        );

        assert!(!relocated.contains("struct Formatter"));
        assert!(relocated.contains("struct ChildFormatter"));
        assert!(relocated.contains("From<ChildFormatter> for Formatter"));
        syn::parse_file(&relocated).expect("retained local-child conversion should parse");
    }
