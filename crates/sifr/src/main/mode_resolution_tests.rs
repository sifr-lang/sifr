    use super::*;
    use std::sync::{Mutex, MutexGuard, OnceLock};

    static CWD_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    struct CurrentDirGuard {
        previous: PathBuf,
        _lock: MutexGuard<'static, ()>,
    }

    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            std::env::set_current_dir(&self.previous).expect("restore cwd");
        }
    }

    fn enter_test_cwd(path: &Path) -> CurrentDirGuard {
        let lock = CWD_TEST_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("cwd test lock should not be poisoned");
        let previous = std::env::current_dir().expect("cwd exists");
        std::env::set_current_dir(path).expect("chdir to test cwd");
        CurrentDirGuard {
            previous,
            _lock: lock,
        }
    }

    fn mktemp_dir(name: &str) -> PathBuf {
        let unique = format!(
            "sifr_cli_mode_{name}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&dir).expect("temp dir should be created");
        dir
    }

    fn resolved_mode(file: &Path) -> CompilationMode {
        resolve_compilation_mode(file).expect("compilation mode should resolve")
    }

    fn test_diagnostic(
        code: &str,
        severity: Severity,
        message: &str,
        span: Option<DiagnosticSpan>,
        help: Option<&str>,
    ) -> RenderedDiagnostic {
        RenderedDiagnostic {
            code: code.to_string(),
            severity,
            message: message.to_string(),
            message_template: "{message}".to_string(),
            args: BTreeMap::new(),
            url: format!("https://sifr.sh/docs/errors/{code}"),
            spans: span.into_iter().collect(),
            children: Vec::new(),
            help: help.map(str::to_string),
            suggestions: Vec::new(),
        }
    }

    fn primary_test_span(file: &str, line: u32, column: u32) -> DiagnosticSpan {
        let byte_start = (line.saturating_sub(1) * 100) + column.saturating_sub(1);
        DiagnosticSpan {
            file: Some(file.to_string()),
            byte_start,
            byte_end: byte_start + 1,
            line: Some(line),
            column: Some(column),
            end_line: Some(line),
            end_column: Some(column),
            is_primary: true,
            label: None,
            lines: Vec::new(),
        }
    }

    #[test]
    fn test_json_diagnostic_format_uses_canonical_rendered_schema() {
        let diagnostics = vec![diagnostic_with_code(
            "sample diagnostic",
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
        )];
        let json = serde_json::to_value(&diagnostics)
            .expect("diagnostics should serialize to canonical JSON");
        let first = json
            .as_array()
            .and_then(|items| items.first())
            .and_then(serde_json::Value::as_object)
            .expect("diagnostic JSON should be an object");

        assert!(first.contains_key("message_template"));
        assert!(first.contains_key("args"));
        assert!(first.contains_key("spans"));
        assert!(!first.contains_key("primary_span"));
        assert!(!first.contains_key("related_spans"));
    }

    struct TestProject {
        dir: PathBuf,
    }

    impl TestProject {
        fn new(name: &str) -> Self {
            Self {
                dir: mktemp_dir(name),
            }
        }

        /// Writes a test fixture and creates any missing parent directories first.
        fn write(&self, relative_path: &str, contents: &str, failure_message: &str) -> PathBuf {
            let path = self.dir.join(relative_path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("test fixture parent should exist");
            }
            std::fs::write(&path, contents).expect(failure_message);
            path
        }
    }

    impl Drop for TestProject {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    fn write_real_sifr_package(
        root: &Path,
        cargo_name: &str,
        sifr_name: &str,
        cargo_dependencies: &str,
    ) {
        std::fs::create_dir_all(root.join("src")).expect("package src dir should exist");
        std::fs::write(root.join("src/lib.rs"), "").expect("pure marker should be written");
        std::fs::write(
            root.join("Cargo.toml"),
            format!(
                "[package]\nname = \"{cargo_name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\n{cargo_dependencies}\n\n[package.metadata.sifr]\nmanifest = \"sifr.toml\"\n"
            ),
        )
        .expect("cargo manifest should be written");
        std::fs::write(
            root.join("sifr.toml"),
            format!(
                "[package]\nname = \"{sifr_name}\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n"
            ),
        )
        .expect("sifr manifest should be written");
    }

    #[test]
    fn test_invocation_workspace_create_returns_unique_paths() {
        let first = InvocationWorkspace::create("sifr_run_workspace")
            .expect("first workspace should exist");
        let second = InvocationWorkspace::create("sifr_run_workspace")
            .expect("second workspace should exist");
        assert_ne!(first.path(), second.path());
        assert!(first.path().exists());
        assert!(second.path().exists());
    }

    #[test]
    fn test_resolve_compilation_mode_project_for_main_with_siblings() {
        let project = TestProject::new("project");
        let main = project.write(
            "main.sifr",
            "from helper import value\n\ndef main():\n    print(value())\n",
            "main file should be written",
        );
        project.write(
            "helper.sifr",
            "def helper() -> int:\n    return 1\n",
            "helper file should be written",
        );

        assert_eq!(resolved_mode(&main), CompilationMode::Project);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_non_main_entry() {
        let project = TestProject::new("single");
        let app = project.write(
            "app.sifr",
            "def main():\n    pass\n",
            "app file should be written",
        );
        project.write(
            "helper.sifr",
            "def helper() -> int:\n    return 1\n",
            "helper file should be written",
        );

        assert_eq!(resolved_mode(&app), CompilationMode::SingleFile);
    }

    #[test]
    fn test_manifest_less_run_explicit_non_main_file_stays_single_file() {
        let project = TestProject::new("manifest_less_non_main");
        let app = project.write(
            "app.sifr",
            "def main():\n    pass\n",
            "app file should be written",
        );
        project.write(
            "main.sifr",
            "from helper import value\n\ndef main():\n    print(value())\n",
            "project-like sibling should be written",
        );

        assert_eq!(resolved_mode(&app), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_project_for_non_main_entry_in_workspace() {
        let project = TestProject::new("workspace_non_main");
        project.write(
            "sifr.toml",
            "[source]\nroots = [\"src\"]\n",
            "manifest should be written",
        );
        project.write(
            "src/helper.sifr",
            "VALUE: int = 1\n",
            "helper should be written",
        );
        let app = project.write(
            "src/app.sifr",
            "from helper import VALUE\n\ndef main():\n    print(VALUE)\n",
            "app file should be written",
        );

        assert_eq!(resolved_mode(&app), CompilationMode::Project);
    }

    #[test]
    fn test_resolve_compilation_mode_reports_malformed_workspace_manifest() {
        let project = TestProject::new("workspace_malformed");
        project.write(
            "sifr.toml",
            "[source\nroots = [\".\"]\n",
            "manifest should be written",
        );
        let app = project.write(
            "app.sifr",
            "def main():\n    pass\n",
            "app should be written",
        );

        let errors = resolve_compilation_mode(&app)
            .expect_err("malformed manifest should prevent single-file fallback");

        assert!(errors[0].message.contains("could not parse sifr.toml"));
    }

    #[test]
    fn test_manifest_less_mode_does_not_ignore_malformed_package_manifest() {
        let project = TestProject::new("manifest_less_malformed_manifest");
        project.write(
            "sifr.toml",
            "[source\nroots = [\".\"]\n",
            "manifest should be written",
        );
        let app = project.write(
            "app.sifr",
            "def main():\n    pass\n",
            "app should be written",
        );

        let errors = resolve_compilation_mode(&app)
            .expect_err("package manifest should prevent manifest-less fallback");

        assert!(errors[0].message.contains("could not parse sifr.toml"));
    }

    #[test]
    fn test_package_cli_init_lib_creates_projection() {
        let dir = mktemp_dir("package_cli_init_lib");
        let package = dir.join("demo_json");

        let exit = cmd_init(
            &package,
            true,
            false,
            Some("demo_json"),
            false,
            DiagnosticFormat::Compact,
        );

        assert_eq!(exit, EXIT_SUCCESS);
        assert!(package.join("sifr.toml").is_file());
        assert!(package.join("Cargo.toml").is_file());
        assert!(package.join("src/__init__.sifr").is_file());
        assert!(package.join("src/lib.rs").is_file());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_package_cli_repair_check_reports_projection_drift() {
        let dir = mktemp_dir("package_cli_repair_check");
        let package = dir.join("demo_json");
        assert_eq!(
            cmd_init(
                &package,
                true,
                false,
                Some("demo_json"),
                false,
                DiagnosticFormat::Compact,
            ),
            EXIT_SUCCESS
        );
        std::fs::write(
            package.join("Cargo.toml"),
            "[package]\nname = \"sifr-demo-json\"\n",
        )
        .expect("break projection");
        let exit = {
            let _cwd = enter_test_cwd(&package);
            cmd_repair(true, DiagnosticFormat::Compact)
        };
        assert_eq!(exit, EXIT_USER_DIAGNOSTIC);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_package_cli_parses_run_script_bin_and_app_args() {
        let cli = Cli::try_parse_from([
            "sifr", "run", "--script", "dev", "--locked", "--", "--port", "8080",
        ])
        .expect("run script cli parses");

        let Some(Commands::Run {
            script,
            locked,
            args,
            ..
        }) = cli.command
        else {
            panic!("expected run command");
        };
        assert_eq!(script.as_deref(), Some("dev"));
        assert!(locked);
        assert_eq!(args, ["--port", "8080"]);

        let cli =
            Cli::try_parse_from(["sifr", "run", "--bin", "admin"]).expect("run bin cli parses");
        let Some(Commands::Run { bin, .. }) = cli.command else {
            panic!("expected run command");
        };
        assert_eq!(bin.as_deref(), Some("admin"));
    }

    #[test]
    fn test_package_cli_check_explicit_file_uses_package_imports() {
        let dir = mktemp_dir("package_cli_check_package_imports");
        let app_root = dir.join("app");
        let json_root = dir.join("json");
        write_real_sifr_package(
            &app_root,
            "sifr-demo-app",
            "demo_app",
            "demo_json = { path = \"../json\", package = \"sifr-demo-json\" }\n",
        );
        let cargo_toml = app_root.join("Cargo.toml");
        let cargo_source =
            std::fs::read_to_string(&cargo_toml).expect("app cargo manifest should be readable");
        std::fs::write(
            &cargo_toml,
            cargo_source.replace(
                "[dependencies]",
                "[package.metadata.sifr.aliases]\ndemo_json_v1 = { dependency = \"demo_json\", import = \"demo_json_v1\" }\n\n[dependencies]",
            ),
        )
        .expect("app cargo manifest should be updated with alias");
        write_real_sifr_package(&json_root, "sifr-demo-json", "demo_json", "");
        std::fs::write(
            app_root.join("src/main.sifr"),
            "from demo_json_v1 import parse_json\n\n\
def main():\n    assert parse_json() == 1\n",
        )
        .expect("app source should be written");
        std::fs::write(
            json_root.join("src/__init__.sifr"),
            "from .parse import parse_json\n",
        )
        .expect("json namespace should be written");
        std::fs::write(
            json_root.join("src/parse.sifr"),
            "def parse_json() -> int:\n    return 1\n",
        )
        .expect("json implementation should be written");
        let exit = {
            let _cwd = enter_test_cwd(&app_root);
            cmd_check(
                Some(Path::new("src/main.sifr")),
                None,
                &sifr_package::CargoPackageSelection::default(),
                sifr_package::CargoLockMode::Normal,
                DiagnosticFormat::Compact,
            )
        };
        assert_eq!(exit, EXIT_SUCCESS);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn test_package_cli_check_explicit_file_falls_back_for_legacy_workspace_manifest() {
        let project = TestProject::new("package_cli_check_legacy_workspace_manifest");
        project.write(
            "Cargo.toml",
            "[workspace]\nmembers = [\"rust_member\"]\nresolver = \"2\"\n",
            "workspace manifest should be written",
        );
        project.write(
            "rust_member/Cargo.toml",
            "[package]\nname = \"rust-member\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            "rust member manifest should be written",
        );
        project.write(
            "rust_member/src/lib.rs",
            "pub fn value() -> i32 { 1 }\n",
            "rust member source should be written",
        );
        project.write(
            "sifr.toml",
            "[package]\nname = \"legacy-workspace\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroots = [\".\"]\n",
            "legacy workspace manifest should be written",
        );
        project.write(
            "helper.sifr",
            "def value() -> int:\n    return 1\n",
            "helper source should be written",
        );
        project.write(
            "main.sifr",
            "from helper import value\n\n\
def main():\n    assert value() == 1\n",
            "main source should be written",
        );
        let exit = {
            let _cwd = enter_test_cwd(&project.dir);
            cmd_check(
                Some(Path::new("main.sifr")),
                None,
                &sifr_package::CargoPackageSelection::default(),
                sifr_package::CargoLockMode::Normal,
                DiagnosticFormat::Compact,
            )
        };
        assert_eq!(exit, EXIT_SUCCESS);
    }

    #[test]
    fn test_package_cli_parses_check_message_format_and_tree_args() {
        let cli = Cli::try_parse_from([
            "sifr",
            "check",
            "--locked",
            "--workspace",
            "-p",
            "demo-app",
            "--exclude",
            "demo-tools",
            "--message-format",
            "json",
        ])
        .expect("check cli parses");
        let Some(Commands::Check {
            message_format,
            locked,
            workspace,
            packages,
            exclude,
            ..
        }) = cli.command
        else {
            panic!("expected check command");
        };
        assert_eq!(message_format.as_deref(), Some("json"));
        assert!(locked);
        assert!(workspace);
        assert_eq!(packages, ["demo-app"]);
        assert_eq!(exclude, ["demo-tools"]);

        let cli = Cli::try_parse_from(["sifr", "tree", "--offline", "--depth", "1"])
            .expect("tree cli parses");
        let Some(Commands::Tree { offline, args, .. }) = cli.command else {
            panic!("expected tree command");
        };
        assert!(offline);
        assert_eq!(args, ["--depth", "1"]);

        let cli = Cli::try_parse_from([
            "sifr",
            "package",
            "--workspace",
            "-p",
            "demo-app",
            "--exclude",
            "demo-tools",
            "--list",
            "--no-verify",
            "--no-metadata",
            "--allow-dirty",
            "--exclude-lockfile",
            "--frozen",
        ])
        .expect("package cli parses");
        let Some(Commands::Package {
            workspace,
            packages,
            exclude,
            list,
            no_verify,
            no_metadata,
            allow_dirty,
            exclude_lockfile,
            frozen,
            ..
        }) = cli.command
        else {
            panic!("expected package command");
        };
        assert!(workspace);
        assert_eq!(packages, ["demo-app"]);
        assert_eq!(exclude, ["demo-tools"]);
        assert!(list);
        assert!(no_verify);
        assert!(no_metadata);
        assert!(allow_dirty);
        assert!(exclude_lockfile);
        assert!(frozen);

        let cli = Cli::try_parse_from([
            "sifr",
            "publish",
            "--dry-run",
            "-p",
            "demo-app",
            "--no-verify",
            "--allow-dirty",
            "--locked",
        ])
        .expect("publish cli parses");
        let Some(Commands::Publish {
            dry_run,
            packages,
            no_verify,
            allow_dirty,
            locked,
            ..
        }) = cli.command
        else {
            panic!("expected publish command");
        };
        assert!(dry_run);
        assert_eq!(packages, ["demo-app"]);
        assert!(no_verify);
        assert!(allow_dirty);
        assert!(locked);

        let cli = Cli::try_parse_from([
            "sifr",
            "vendor",
            "third_party/vendor",
            "--sync",
            "member/Cargo.toml",
            "--no-delete",
            "--respect-source-config",
            "--versioned-dirs",
            "--offline",
        ])
        .expect("vendor cli parses");
        let Some(Commands::Vendor {
            path,
            sync,
            no_delete,
            respect_source_config,
            versioned_dirs,
            offline,
            ..
        }) = cli.command
        else {
            panic!("expected vendor command");
        };
        assert_eq!(path, PathBuf::from("third_party/vendor"));
        assert_eq!(sync, [PathBuf::from("member/Cargo.toml")]);
        assert!(no_delete);
        assert!(respect_source_config);
        assert!(versioned_dirs);
        assert!(offline);
    }

    #[test]
    fn test_package_cli_explain_retired_credential_code() {
        let text =
            diagnostic_explanation("SIFR-PACKAGE-0105").expect("retired code explanation exists");
        assert!(text.contains("retired"));
        assert!(text.contains("SIFR-PACKAGE-0101"));
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_main_without_local_imports() {
        let project = TestProject::new("main_no_imports");
        let main = project.write(
            "main.sifr",
            "def main():\n    print(\"ok\")\n",
            "main file should be written",
        );
        project.write(
            "scratch.sifr",
            "def nope(:\n",
            "scratch file should be written",
        );

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_stdlib_only_imports() {
        let project = TestProject::new("main_stdlib_only");
        let main = project.write(
            "main.sifr",
            "from sifr.math import floor\n\ndef main():\n    print(floor(3.9))\n",
            "main file should be written",
        );
        project.write(
            "helper.sifr",
            "def helper() -> int:\n    return 1\n",
            "helper file should be written",
        );

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_missing_local_module() {
        let project = TestProject::new("missing_local");
        let main = project.write(
            "main.sifr",
            "from helper import value\n\ndef main():\n    print(value())\n",
            "main file should be written",
        );

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_regular_import_with_local_module() {
        let project = TestProject::new("regular_import_local_module");
        let main = project.write(
            "main.sifr",
            "import helper\n\ndef main():\n    print(\"ok\")\n",
            "main file should be written",
        );
        project.write(
            "helper.sifr",
            "def value() -> int:\n    return 1\n",
            "helper file should be written",
        );

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_invalid_main_source() {
        let project = TestProject::new("invalid_main");
        let main = project.write("main.sifr", "def main(:\n", "main file should be written");
        project.write(
            "helper.sifr",
            "def helper() -> int:\n    return 1\n",
            "helper file should be written",
        );

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_typing_import() {
        let project = TestProject::new("typing_import");
        let main = project.write(
            "main.sifr",
            "from typing import List\n\ndef main():\n    values: List[int] = [1]\n    print(values)\n",
            "main file should be written",
        );
        project.write(
            "helper.sifr",
            "def helper() -> int:\n    return 1\n",
            "helper file should be written",
        );

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_typing_import_with_local_typing_file() {
        let project = TestProject::new("typing_import_local_file");
        let main = project.write(
            "main.sifr",
            "from typing import List\n\ndef main():\n    values: List[int] = [1]\n    print(values)\n",
            "main file should be written",
        );
        project.write(
            "typing.sifr",
            "def local() -> int:\n    return 1\n",
            "typing file should be written",
        );

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_enum_import() {
        let project = TestProject::new("enum_import");
        let main = project.write(
            "main.sifr",
            "from enum import Enum\n\ndef main():\n    print(\"ok\")\n",
            "main file should be written",
        );
        project.write(
            "helper.sifr",
            "def helper() -> int:\n    return 1\n",
            "helper file should be written",
        );

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_enum_import_with_local_enum_file() {
        let project = TestProject::new("enum_import_local_file");
        let main = project.write(
            "main.sifr",
            "from enum import Enum\n\ndef main():\n    print(\"ok\")\n",
            "main file should be written",
        );
        project.write(
            "enum.sifr",
            "def local() -> int:\n    return 1\n",
            "enum file should be written",
        );

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_package_init_import() {
        let project = TestProject::new("pkg_import");
        let main = project.write(
            "main.sifr",
            "from pkg import value\n\ndef main():\n    print(value())\n",
            "main file should be written",
        );
        project.write(
            "pkg/__init__.sifr",
            "def value() -> int:\n    return 1\n",
            "pkg init should be written",
        );

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
    }

    #[test]
    fn test_resolve_compilation_mode_project_for_relative_import_with_sibling() {
        let project = TestProject::new("relative_import");
        let main = project.write(
            "main.sifr",
            "from .helper import value\n\ndef main():\n    print(value())\n",
            "main file should be written",
        );
        project.write(
            "helper.sifr",
            "def value() -> int:\n    return 1\n",
            "helper file should be written",
        );

        assert_eq!(resolved_mode(&main), CompilationMode::Project);
    }

    #[test]
    fn test_resolve_compilation_mode_single_file_for_relative_import_without_sibling() {
        let project = TestProject::new("relative_import_missing_sibling");
        let main = project.write(
            "main.sifr",
            "from .helper import value\n\ndef main():\n    print(value())\n",
            "main file should be written",
        );

        assert_eq!(resolved_mode(&main), CompilationMode::SingleFile);
    }

