use super::project_codegen::GeneratedBinaryProject;
use super::rust_interop::{
    PackageRustInteropContext, RustInteropModuleSource, apply_package_rust_interop_metadata,
};
use super::rust_interop_contract_tests::{
    backend_with_manifest, interop_errors, package_context, param_contract, result_contract,
    set_bridge_roots, signature_contract, string_contract, temp_package_root,
};
use sifr_codegen::{
    RustBridgeParamConvention, RustBridgeSignatureContract, RustInteropOwner, StdlibCode,
    generate_rust_multi_with_metadata,
};
use sifr_package::TrustPolicy;
use std::collections::BTreeMap;
use std::path::PathBuf;

const ASYNC_HASH_SOURCE: &str = r#"
class RustError(Error):
    message: str

@rust(native.hash)
async def hash(input: str) -> Result[str, RustError | RustPanicError]:
    await task.sleep(0.0)
    return "ok"
"#;

const ASYNC_HASH_CURRENT_THREAD_SOURCE: &str = r#"
class RustError(Error):
    message: str

@rust.async(thread_affinity=tokio_current_thread)
@rust(native.hash)
async def hash(input: str) -> Result[str, RustError | RustPanicError]:
    await task.sleep(0.0)
    return "ok"
"#;

const ASYNC_BORROWED_SOURCE: &str = r#"
class RustError(Error):
    message: str

@rust(native.hash)
async def hash(input: str) -> Result[str, RustError | RustPanicError]:
    await task.sleep(0.0)
    return "ok"
"#;

#[test]
fn package_rust_interop_direct_probe_accepts_async_signature() {
    let backend_root = async_backend_root(
        "rust_interop_async_signature_probe_ok",
        "pub async fn hash(input: String) -> Result<String, String> { Ok(input) }\n",
    );
    let generated = generated_from_source(ASYNC_HASH_SOURCE, vec![hash_signature_contract()]);
    let context = context_with_source(
        ASYNC_HASH_SOURCE,
        vec![backend_with_manifest(
            "native",
            backend_root.join("Cargo.toml"),
        )],
    );

    apply_package_rust_interop_metadata(generated, Some(context))
        .expect("compatible async signature should pass probe");
}

#[test]
fn package_rust_interop_direct_probe_accepts_async_borrowed_signature() {
    let backend_root = async_backend_root(
        "rust_interop_async_borrowed_signature_probe_ok",
        "pub async fn hash(input: &str) -> Result<String, String> { Ok(input.to_string()) }\n",
    );
    let generated = generated_from_source(
        ASYNC_BORROWED_SOURCE,
        vec![borrowed_hash_signature_contract()],
    );
    let context = context_with_source(
        ASYNC_BORROWED_SOURCE,
        vec![backend_with_manifest(
            "native",
            backend_root.join("Cargo.toml"),
        )],
    );

    apply_package_rust_interop_metadata(generated, Some(context))
        .expect("async borrowed signature should preserve its input lifetime in the probe");
}

#[test]
fn package_rust_interop_direct_probe_rejects_static_only_async_borrow() {
    let backend_root = async_backend_root(
        "rust_interop_async_static_borrow_probe_bad",
        "pub async fn hash(input: &'static str) -> Result<String, String> { Ok(input.to_string()) }\n",
    );
    let generated = generated_from_source(
        ASYNC_BORROWED_SOURCE,
        vec![borrowed_hash_signature_contract()],
    );
    let context = context_with_source(
        ASYNC_BORROWED_SOURCE,
        vec![backend_with_manifest(
            "native",
            backend_root.join("Cargo.toml"),
        )],
    );

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "static-only async borrow must fail the caller-lifetime probe",
    );

    assert_eq!(diagnostics[0].code, "SIFR-RUST-TYPE-0001");
    assert!(diagnostics[0].message.contains("Rust bridge probe failed"));
}

#[test]
fn package_rust_interop_direct_probe_rejects_sync_function_for_async_binding() {
    let backend_root = async_backend_root(
        "rust_interop_async_signature_probe_bad",
        "pub fn hash(input: String) -> Result<String, String> { Ok(input) }\n",
    );
    let generated = generated_from_source(ASYNC_HASH_SOURCE, vec![hash_signature_contract()]);
    let context = context_with_source(
        ASYNC_HASH_SOURCE,
        vec![backend_with_manifest(
            "native",
            backend_root.join("Cargo.toml"),
        )],
    );

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "sync function must fail async probe",
    );

    assert_eq!(diagnostics[0].code, "SIFR-RUST-TYPE-0001");
    assert!(diagnostics[0].message.contains("Rust bridge probe failed"));
}

#[test]
fn package_rust_interop_async_requires_send_future_by_default() {
    let backend_root = async_backend_root(
        "rust_interop_async_non_send_default",
        non_send_future_backend(),
    );
    let generated = generated_from_source(ASYNC_HASH_SOURCE, vec![hash_signature_contract()]);
    let context = context_with_source(
        ASYNC_HASH_SOURCE,
        vec![backend_with_manifest(
            "native",
            backend_root.join("Cargo.toml"),
        )],
    );

    let diagnostics = interop_errors(generated, Some(context), "non-Send future must fail");

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ASYNC-0001");
    assert!(diagnostics[0].message.contains("must be Send"));
}

#[test]
#[doc = "sifr-evidence: executes-cargo-probe"]
fn package_rust_interop_async_probe_current_thread_allows_non_send_future() {
    let backend_root = async_backend_root(
        "rust_interop_async_non_send_current_thread",
        non_send_future_backend(),
    );
    let generated = generated_from_source(
        ASYNC_HASH_CURRENT_THREAD_SOURCE,
        vec![hash_signature_contract()],
    );
    let context = context_with_source(
        ASYNC_HASH_CURRENT_THREAD_SOURCE,
        vec![backend_with_manifest(
            "native",
            backend_root.join("Cargo.toml"),
        )],
    );

    apply_package_rust_interop_metadata(generated, Some(context))
        .expect("current-thread async Rust target should allow non-Send future");
}

#[test]
fn package_rust_interop_async_rejects_unsupported_thread_affinity() {
    let source = r#"
class RustError(Error):
    message: str

@rust.async(thread_affinity=current_os_thread)
@rust(native.hash)
async def hash(input: str) -> Result[str, RustError | RustPanicError]:
    await task.sleep(0.0)
    return "ok"
"#;
    let generated = generated_from_source(source, vec![hash_signature_contract()]);
    let context = context_with_source(
        source,
        vec![backend_with_manifest(
            "native",
            PathBuf::from("/ws/native/Cargo.toml"),
        )],
    );

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "unsupported async thread affinity must fail",
    );

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ASYNC-0001");
    assert!(
        diagnostics[0]
            .message
            .contains("thread_affinity=` must be none or tokio_current_thread")
    );
}

#[test]
fn package_rust_interop_async_rejects_hidden_nested_runtime_operations() {
    let source = r#"
class RustError(Error):
    message: str

@rust(bridge.http.fetch, panic=trusted_no_panic)
async def fetch(url: str) -> Result[str, RustError]:
    await task.sleep(0.0)
    return ""
"#;
    let package_root = temp_package_root("rust_interop_async_hidden_runtime");
    std::fs::create_dir_all(package_root.join("rust/interop")).expect("create bridge source");
    std::fs::write(
        package_root.join("Cargo.toml"),
        "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write package manifest");
    std::fs::write(
        package_root.join("rust/interop/http.rs"),
        r#"
pub async fn fetch(_url: &str) -> Result<String, String> {
    let control = "runtime.block_on and Builder::new_multi_thread are ignored in strings";
    // Runtime::new and block_on in comments are also ignored.
    let _thread = std::thread::Builder::new();
    Ok(control.to_string())
}
"#,
    )
    .expect("write bridge source");
    std::fs::write(
        package_root.join("rust/interop/runtime_helper.rs"),
        r#"
use futures::executor::block_on as wait;
use tokio::{runtime::Builder as Rt, task::block_in_place};

pub fn hidden_runtime_operations() {
    let runtime = Rt::new_current_thread();
    runtime.block_on(async {});
    wait(async {});
    block_in_place(|| {});
}
"#,
    )
    .expect("write forbidden helper source");
    let generated = generated_from_source(source, Vec::new());
    let mut context = context_with_source(source, Vec::new());
    set_bridge_roots(&mut context, vec![PathBuf::from("rust/interop")]);
    let package = context
        .graph
        .packages
        .get_mut(&context.package_id)
        .expect("package metadata");
    package.package_root = package_root;

    let diagnostics = interop_errors(
        generated,
        Some(context),
        "nested bridge runtime operations must fail before Cargo probing",
    );

    assert_eq!(diagnostics[0].code, "SIFR-RUST-ASYNC-0001");
    assert!(
        diagnostics[0]
            .message
            .contains("reuse the generated Tokio runtime")
    );
    assert!(
        diagnostics[0]
            .children
            .iter()
            .any(|child| child.message.contains("block_on")
                && child.message.contains("Tokio runtime construction")
                && child.message.contains("blocking runtime operation")
                && child.message.contains("rust/interop/runtime_helper.rs"))
    );
}

#[test]
fn package_rust_interop_opaque_current_thread_clears_async_method_send_probe() {
    let source = r#"
class RustError(Error):
    message: str

@rust.opaque(type=bridge.Client, thread_affinity=tokio_current_thread)
class Client:
    @rust(Self.hash)
    async def hash(self, input: str) -> Result[str, RustError | RustPanicError]:
        return "ok"
"#;
    let mut generated = generated_from_source(source, vec![client_hash_signature_contract()]);
    let mut context = context_with_source(source, Vec::new());
    set_bridge_roots(&mut context, vec![PathBuf::from("src/bridges")]);

    generated = apply_package_rust_interop_metadata(generated, Some(context))
        .expect("opaque current-thread async method should resolve");

    let method_probe = generated
        .interop
        .rust
        .probe_plan
        .probes
        .iter()
        .find(|probe| probe.canonical_target_path == "app.Client.hash")
        .expect("async method probe");
    assert!(!method_probe.requires_send);
}

fn generated_from_source(
    source: &str,
    signatures: Vec<RustBridgeSignatureContract>,
) -> GeneratedBinaryProject {
    let parsed = sifr_syntax::parse_module(source, Some("app")).expect("source should parse");
    let module = sifr_lowering::lower_module(parsed.suite())
        .map(|result| result.module)
        .expect("source should lower");
    let mut result = generate_rust_multi_with_metadata(&[("app", &module)], &StdlibCode::default());
    result.interop.rust.bridge_contracts.signatures = signatures;
    let main_rs = result.rust_files.remove("app").unwrap_or_default();
    GeneratedBinaryProject {
        main_rs,
        support_modules: BTreeMap::new(),
        used_stdlib_modules: result.used_stdlib_modules,
        required_features: result.required_features,
        interop: result.interop,
        cache_key_fragment: None,
        bridge_modules: Default::default(),
        python_runtime: None,
    }
}

fn context_with_source(
    source: &str,
    backend_crates: Vec<sifr_package::BackendCrateMetadata>,
) -> PackageRustInteropContext {
    let mut context = package_context(TrustPolicy::default(), backend_crates);
    context.module_sources.insert(
        "app".to_string(),
        RustInteropModuleSource {
            source: source.to_string(),
            display_path: "/ws/app/sifr/app.sifr".to_string(),
        },
    );
    context
}

fn async_backend_root(name: &str, lib_rs: &str) -> PathBuf {
    let backend_root = temp_package_root(name);
    std::fs::create_dir_all(backend_root.join("src")).expect("create backend src");
    std::fs::write(
        backend_root.join("Cargo.toml"),
        "[package]\nname = \"native\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write backend cargo toml");
    std::fs::write(backend_root.join("src/lib.rs"), lib_rs).expect("write backend lib");
    backend_root
}

fn hash_signature_contract() -> RustBridgeSignatureContract {
    let mut error = string_contract();
    error.sifr_type = "RustError | RustPanicError".to_string();
    let mut signature = signature_contract(
        vec![param_contract(
            "input",
            RustBridgeParamConvention::Own,
            string_contract(),
        )],
        result_contract(string_contract(), error),
    );
    signature.panic_error = sifr_codegen::RustBridgePanicErrorContract::OrdinaryAndWrapper;
    signature
}

fn borrowed_hash_signature_contract() -> RustBridgeSignatureContract {
    let mut signature = hash_signature_contract();
    signature.params[0].convention = RustBridgeParamConvention::Borrow;
    signature
}

fn client_hash_signature_contract() -> RustBridgeSignatureContract {
    let mut signature = hash_signature_contract();
    signature.canonical_target_path = "app.Client.hash".to_string();
    signature.owner = RustInteropOwner::Method {
        class_name: "Client".to_string(),
        name: "hash".to_string(),
    };
    signature
}

fn non_send_future_backend() -> &'static str {
    "pub async fn hash(input: String) -> Result<String, String> {\n    let local = std::rc::Rc::new(input);\n    std::future::ready(()).await;\n    Ok((*local).clone())\n}\n"
}
