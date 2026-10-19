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
