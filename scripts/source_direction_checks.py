"""Regression fixtures for split test ownership and provider composition roots."""
from pathlib import Path
import tempfile


def check_split_sources_and_composition_roots(guard, failures: list[str]) -> None:
    def fixture(root: Path):
        guard.seed_valid_repo(root)
        src = root / "crates/sifr_codegen/src"
        (src / "lib.rs").write_text('#[cfg(test)]\nmod authority_tests;\n')
        (src / "authority_tests.rs").write_text('include!("fragments/outer.rs");\n')
        (src / "fragments").mkdir()
        (src / "fragments/outer.rs").write_text('include!("inner.rs");\n')
        (src / "fragments/inner.rs").write_text('use sifr_lowering::lower_module;\n')
        return src

    guard.SELF_TEST_TMP_ROOT.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(dir=guard.SELF_TEST_TMP_ROOT) as tmp:
        root = Path(tmp)
        src = fixture(root)
        actual = guard.validate(root)
        if actual:
            failures.append(f"test-owned split includes rejected: {actual}")
        # Shared fragments are production too, regardless of test reachability.
        for source in ('include!("fragments/outer.rs");\n',
                       '#[allow(dead_code)] include!("fragments/outer.rs");\n',
                       'mod fragments { mod inner; }\n',
                       'mod fragments { mod r#inner; }\n',
                       '#[cfg_attr(windows, path = "fragments/outer.rs")]\nmod production;\n',
                       'mod fragments { #[path = "outer.rs"] mod production; }\n',
                       'mod fragments { #[path = r#"outer.rs"#] mod production; }\n',
                       'include!(concat!("fragments/", "outer.rs"));\n',
                       '#[path = "fragments/outer.rs"]\nmod production;\n',
                       '#[path = "fragments/outer.rs"]\n#[allow(dead_code)]\npub(crate) mod production;\n'):
            (src / "lib.rs").write_text(source)
            actual = guard.validate(root)
            if not any('fragments/inner.rs references sifr_lowering' in x for x in actual):
                failures.append(f"production-shared include escaped scanning: {source!r} {actual}")
        (src / "lib.rs").write_text('mod fragments;\n')
        (src / "fragments/mod.rs").write_text('mod inner;\n')
        actual = guard.validate(root)
        if not any('fragments/inner.rs references sifr_lowering' in x for x in actual):
            failures.append(f"ordinary production module escaped scanning: {actual}")

    with tempfile.TemporaryDirectory(dir=guard.SELF_TEST_TMP_ROOT) as tmp:
        root = Path(tmp)
        guard.seed_valid_repo(root)
        for relative in guard.PROVIDER_COMPOSITION_ROOTS:
            path = root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('pub fn compose() { let _ = DiskSourceProvider::new(); }\n')
        actual = guard.validate(root)
        if actual:
            failures.append(f"registered provider composition roots rejected: {actual}")
        for crate in ('sifr_driver', 'sifr_compiler_services'):
            path = root / 'crates' / crate / 'src/lower_service.rs'
            path.write_text('pub fn lower() { let _ = DiskSourceProvider::new(); }\n')
            actual = guard.validate(root)
            if not any(str(path.relative_to(root)) in x and 'outside a composition root' in x for x in actual):
                failures.append(f"lower provider construction escaped: {crate}: {actual}")
            path.unlink()


def check_data_owner_boundaries(guard, failures: list[str]) -> None:
    assert_self_test_case = guard.assert_self_test_case
    write_manifest = guard.write_manifest
    assert_self_test_case(
        "shared stdlib data upward dependency",
        lambda root: write_manifest(root / "crates/sifr_stdlib_artifact",
                                    "sifr_stdlib_artifact", ["sifr_codegen"]),
        "sifr_stdlib_artifact: unexpected normal dependency",
        failures,
    )
    assert_self_test_case(
        "shared stdlib data upward source reference",
        lambda root: (root / "crates/sifr_stdlib_artifact/src/lib.rs").write_text(
            "use sifr_compiler_services::CompilerContext;\n", encoding="utf-8"),
        "sifr_stdlib_artifact: crates/sifr_stdlib_artifact/src/lib.rs references sifr_compiler_services",
        failures,
    )
    for consumer in ("sifr_sysroot", "sifr_stdlib_manifest", "sifr_lowering", "sifr_codegen", "sifr_frontend"):
        assert_self_test_case(
            f"{consumer} raw metadata dependency",
            lambda root, consumer=consumer: write_manifest(
                root / "crates" / consumer, consumer, ["sifr_metadata"]),
            f"{consumer}: forbidden dependency", failures,
        )
        assert_self_test_case(
            f"{consumer} raw metadata re-export",
            lambda root, consumer=consumer: (root / "crates" / consumer / "src/lib.rs").write_text(
                "pub use sifr_metadata::*;\n"),
            f"{consumer}: crates/{consumer}/src/lib.rs references sifr_metadata", failures,
        )
    assert_self_test_case(
        "metadata reverse compiler dependency",
        lambda root: write_manifest(root / "crates/sifr_metadata", "sifr_metadata", ["sifr_codegen"]),
        "sifr_metadata: unexpected normal dependency", failures,
    )
    assert_self_test_case(
        "metadata reverse compiler source reference",
        lambda root: (root / "crates/sifr_metadata/src/lib.rs").write_text(
            "use sifr_compiler_services::CompilerContext;\n"),
        "sifr_metadata: crates/sifr_metadata/src/lib.rs references sifr_compiler_services", failures,
    )

    def alias_edge(root, consumer, dependency):
        path = root / "crates" / consumer / "Cargo.toml"
        with path.open("a") as manifest:
            manifest.write('\n[target.\'cfg(unix)\'.dev-dependencies]\n'
                           f'wire_alias = {{ package = "{dependency}", path = "../{dependency}" }}\n')
    for consumer, dependency in [(name, "sifr_metadata") for name in
            ("sifr_sysroot", "sifr_stdlib_manifest", "sifr_lowering", "sifr_codegen", "sifr_frontend")] + [("sifr_metadata", "sifr_codegen")]:
        assert_self_test_case(
            f"{consumer} target-specific aliased dev edge",
            lambda root, consumer=consumer, dependency=dependency: alias_edge(root, consumer, dependency),
            f"{consumer}: forbidden dependency", failures,
        )
