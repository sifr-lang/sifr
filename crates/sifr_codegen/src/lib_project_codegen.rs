use super::{
    generate_rust, generate_rust_with_stdlib, module_class_fields, module_func_signatures,
    publicize_generated_module_source, sifr_runtime_dependency_spec, tokio_dependency_spec,
    BTreeSet, HashMap, HashSet, HirModule, MultiModuleCodegenResult, Renderer, RustFile, RustItem,
    StdlibCode,
};
pub(super) fn render_local_module_imports(module: &HirModule) -> String {
    let mut module_import_items: Vec<RustItem> = Vec::new();
    for import in &module.imports {
        if import.module.starts_with("sifr.") || import.module.starts_with("_sifr.") {
            continue;
        }
        let mut module_path = vec!["crate".to_string()];
        module_path.extend(import.module.split('.').map(str::to_string));
        for name in &import.names {
            if let Some((_, alias)) = import.aliases.iter().find(|(orig, _)| orig == name) {
                let mut alias_path = module_path.clone();
                alias_path.push(name.clone());
                module_import_items.push(RustItem::UseAlias {
                    path: alias_path,
                    alias: alias.clone(),
                });
            } else {
                let mut import_path = module_path.clone();
                import_path.push(name.clone());
                module_import_items.push(RustItem::Use(import_path));
            }
        }
    }

    if module_import_items.is_empty() {
        String::new()
    } else {
        Renderer::new().render_file(&RustFile {
            items: module_import_items,
        })
    }
}

/// Generate Rust source code for a multi-module project, returning aggregate dependency metadata.
pub fn generate_rust_multi_with_metadata(
    modules: &[(&str, &HirModule)],
    stdlib_code: &StdlibCode,
) -> MultiModuleCodegenResult {
    let mut files = HashMap::new();
    let mut used_stdlib_modules = HashSet::new();
    let mut required_crates = HashSet::new();
    let mut project_codegen_code = stdlib_code.clone();

    for (module_name, module) in modules {
        project_codegen_code
            .func_signatures
            .insert((*module_name).to_string(), module_func_signatures(module));
        project_codegen_code
            .module_class_fields
            .insert((*module_name).to_string(), module_class_fields(module));
    }

    for (module_name, module) in modules {
        let module_public = *module_name != "main";
        let codegen_result = generate_rust_with_stdlib(module, &project_codegen_code);
        let local_imports = render_local_module_imports(module);
        let mut rust_source = codegen_result.rust_source;
        if !local_imports.trim().is_empty() {
            rust_source = format!("{}\n\n{}", local_imports.trim_end(), rust_source);
        }
        if module_public {
            rust_source = publicize_generated_module_source(&rust_source);
        }

        files.insert((*module_name).to_string(), rust_source);
        used_stdlib_modules.extend(codegen_result.used_stdlib_modules);
        required_crates.extend(codegen_result.required_crates);
    }

    MultiModuleCodegenResult {
        rust_files: files,
        used_stdlib_modules,
        required_crates,
    }
}

/// Generate Rust source code for a multi-module project.
/// Returns a map of filename -> Rust source code.
pub fn generate_rust_multi(modules: &[(&str, &HirModule)]) -> HashMap<String, String> {
    generate_rust_multi_with_metadata(modules, &StdlibCode::default())
        .rust_files
        .into_iter()
        .collect()
}

/// Generate a complete Rust project (Cargo.toml + main.rs content).
pub fn generate_project(module: &HirModule, project_name: &str) -> (String, String) {
    generate_project_with_deps(module, project_name, &HashSet::new())
}

/// Generate a complete Rust project with stdlib dependencies.
pub fn generate_project_with_deps(
    module: &HirModule,
    project_name: &str,
    stdlib_modules: &HashSet<String>,
) -> (String, String) {
    generate_project_with_deps_and_crates(module, project_name, stdlib_modules, &HashSet::new())
}

/// Generate a complete Rust project with stdlib and explicit crate dependencies.
pub fn generate_project_with_deps_and_crates(
    module: &HirModule,
    project_name: &str,
    stdlib_modules: &HashSet<String>,
    required_crates: &HashSet<String>,
) -> (String, String) {
    let mut cargo_toml = format!(
        r#"[package]
name = "{project_name}"
version = "0.1.0"
edition = "2021"

[workspace]
"#
    );

    // Add dependencies based on used stdlib/intrinsic modules
    let mut deps = Vec::new();
    for module_name in stdlib_modules
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
    {
        match module_name {
            "sifr.json" | "sifr.collections" | "_sifr.json" | "_sifr.collections" => {
                if !deps.contains(
                    &"serde_json = { version = \"1.0.149\", features = [\"preserve_order\"] }"
                        .to_string(),
                ) {
                    deps.push(
                        "serde_json = { version = \"1.0.149\", features = [\"preserve_order\"] }"
                            .to_string(),
                    );
                    deps.push(
                        "serde = { version = \"1.0.228\", features = [\"derive\"] }".to_string(),
                    );
                }
            }
            "sifr.time" | "_sifr.time" => {
                if !deps.contains(&"chrono = \"0.4.44\"".to_string()) {
                    deps.push("chrono = \"0.4.44\"".to_string());
                }
            }
            "sifr.random" | "_sifr.crypto" => {
                if !deps.contains(&"rand = \"0.10.1\"".to_string()) {
                    deps.push("rand = \"0.10.1\"".to_string());
                }
                if !deps.contains(&"rand_distr = \"0.6.0\"".to_string()) {
                    deps.push("rand_distr = \"0.6.0\"".to_string());
                }
            }
            "sifr.uuid" | "_sifr.uuid" => {
                if !deps.contains(&"rand = \"0.10.1\"".to_string()) {
                    deps.push("rand = \"0.10.1\"".to_string());
                }
                let uuid_dep =
                    "uuid = { version = \"1.23.1\", features = [\"v3\", \"v5\"] }".to_string();
                if !deps.contains(&uuid_dep) {
                    deps.push(uuid_dep);
                }
            }
            "sifr.re" | "_sifr.regex" => {
                if !deps.contains(&"regex = \"1.12.3\"".to_string()) {
                    deps.push("regex = \"1.12.3\"".to_string());
                }
            }
            "sifr.pathlib" => {
                if !deps.contains(&"regex = \"1.12.3\"".to_string()) {
                    deps.push("regex = \"1.12.3\"".to_string());
                }
            }
            "sifr.hash" | "sifr.hashlib" => {
                if !deps.contains(&"sha2 = \"0.11.0\"".to_string()) {
                    deps.push("sha2 = \"0.11.0\"".to_string());
                    deps.push("md5 = \"0.8.0\"".to_string());
                    deps.push("sha1 = \"0.11.0\"".to_string());
                    deps.push("blake2 = \"0.10.6\"".to_string());
                }
            }
            "sifr.encoding" | "sifr.base64" => {
                if !deps.contains(&"base64 = \"0.22.1\"".to_string()) {
                    deps.push("base64 = \"0.22.1\"".to_string());
                }
            }
            "sifr.tomllib" | "_sifr.toml" => {
                let toml_dep =
                    "toml = { version = \"1.1.2\", features = [\"preserve_order\"] }".to_string();
                if !deps.contains(&toml_dep) {
                    deps.push(toml_dep);
                }
            }
            "sifr.datetime" | "_sifr.datetime" => {
                if !deps.contains(&"chrono = \"0.4.44\"".to_string()) {
                    deps.push("chrono = \"0.4.44\"".to_string());
                }
            }
            "sifr.gzip" | "sifr.zipfile" | "_sifr.compress" => {
                if !deps.contains(&"flate2 = \"1.1.9\"".to_string()) {
                    deps.push("flate2 = \"1.1.9\"".to_string());
                }
                if !deps.contains(&"zip = \"8.6.0\"".to_string()) {
                    deps.push("zip = \"8.6.0\"".to_string());
                }
            }
            "_bigint" => {
                if !deps.contains(&"num-bigint = \"0.4.6\"".to_string()) {
                    deps.push("num-bigint = \"0.4.6\"".to_string());
                    deps.push("num-traits = \"0.2.19\"".to_string());
                }
            }
            // sifr.io, sifr.env, sifr.os, sifr.math, sifr.test, sifr.bytes, sifr.sys,
            // sifr.subprocess, sifr.html, sifr.calendar, sifr.operator use only std library
            _ => {}
        }
    }

    for crate_name in required_crates
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
    {
        match crate_name {
            "serde_json" => {
                if !deps.contains(
                    &"serde_json = { version = \"1.0.149\", features = [\"preserve_order\"] }"
                        .to_string(),
                ) {
                    deps.push(
                        "serde_json = { version = \"1.0.149\", features = [\"preserve_order\"] }"
                            .to_string(),
                    );
                }
                if !deps
                    .contains(&"serde = { version = \"1.0.228\", features = [\"derive\"] }".to_string())
                {
                    deps.push("serde = { version = \"1.0.228\", features = [\"derive\"] }".to_string());
                }
            }
            "chrono" => {
                if !deps.contains(&"chrono = \"0.4.44\"".to_string()) {
                    deps.push("chrono = \"0.4.44\"".to_string());
                }
            }
            "rand" => {
                if !deps.contains(&"rand = \"0.10.1\"".to_string()) {
                    deps.push("rand = \"0.10.1\"".to_string());
                }
            }
            "rand_distr" => {
                if !deps.contains(&"rand_distr = \"0.6.0\"".to_string()) {
                    deps.push("rand_distr = \"0.6.0\"".to_string());
                }
            }
            "regex" => {
                if !deps.contains(&"regex = \"1.12.3\"".to_string()) {
                    deps.push("regex = \"1.12.3\"".to_string());
                }
            }
            "sha2" => {
                if !deps.contains(&"sha2 = \"0.11.0\"".to_string()) {
                    deps.push("sha2 = \"0.11.0\"".to_string());
                }
            }
            "md5" => {
                if !deps.contains(&"md5 = \"0.8.0\"".to_string()) {
                    deps.push("md5 = \"0.8.0\"".to_string());
                }
            }
            "sha1" => {
                if !deps.contains(&"sha1 = \"0.11.0\"".to_string()) {
                    deps.push("sha1 = \"0.11.0\"".to_string());
                }
            }
            "uuid" => {
                let uuid_dep =
                    "uuid = { version = \"1.23.1\", features = [\"v3\", \"v5\"] }".to_string();
                if !deps.contains(&uuid_dep) {
                    deps.push(uuid_dep);
                }
            }
            "blake2" => {
                if !deps.contains(&"blake2 = \"0.10.6\"".to_string()) {
                    deps.push("blake2 = \"0.10.6\"".to_string());
                }
            }
            "base64" => {
                if !deps.contains(&"base64 = \"0.22.1\"".to_string()) {
                    deps.push("base64 = \"0.22.1\"".to_string());
                }
            }
            "toml" => {
                let toml_dep =
                    "toml = { version = \"1.1.2\", features = [\"preserve_order\"] }".to_string();
                if !deps.contains(&toml_dep) {
                    deps.push(toml_dep);
                }
            }
            "flate2" => {
                if !deps.contains(&"flate2 = \"1.1.9\"".to_string()) {
                    deps.push("flate2 = \"1.1.9\"".to_string());
                }
            }
            "zip" => {
                if !deps.contains(&"zip = \"8.6.0\"".to_string()) {
                    deps.push("zip = \"8.6.0\"".to_string());
                }
            }
            "num-bigint" => {
                if !deps.contains(&"num-bigint = \"0.4.6\"".to_string()) {
                    deps.push("num-bigint = \"0.4.6\"".to_string());
                }
            }
            "num-traits" => {
                if !deps.contains(&"num-traits = \"0.2.19\"".to_string()) {
                    deps.push("num-traits = \"0.2.19\"".to_string());
                }
            }
            "rust_decimal" => {
                if !deps.contains(
                    &"rust_decimal = { version = \"1.41.0\", features = [\"maths\", \"serde-with-str\"] }".to_string(),
                ) {
                    deps.push(
                        "rust_decimal = { version = \"1.41.0\", features = [\"maths\", \"serde-with-str\"] }".to_string(),
                    );
                }
            }
            "bigdecimal" => {
                if !deps.contains(
                    &"bigdecimal = { version = \"0.4.10\", features = [\"serde\"] }".to_string(),
                ) {
                    deps.push(
                        "bigdecimal = { version = \"0.4.10\", features = [\"serde\"] }".to_string(),
                    );
                }
            }
            "sifr_runtime" | "sifr-runtime" => {
                let dep = sifr_runtime_dependency_spec();
                if !deps.contains(&dep) {
                    deps.push(dep);
                }
            }
            "tokio" => {
                let dep = tokio_dependency_spec();
                if !deps.contains(&dep) {
                    deps.push(dep);
                }
            }
            _ => {}
        }
    }

    if !deps.is_empty() {
        cargo_toml.push_str("\n[dependencies]\n");
        for dep in &deps {
            cargo_toml.push_str(dep);
            cargo_toml.push('\n');
        }
    }

    let main_rs = generate_rust(module);
    (cargo_toml, main_rs)
}
