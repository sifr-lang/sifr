use super::project_build_check::mktemp_dir;
use crate::{build_project, check_project};

#[test]
fn test_check_project_imports_generic_function_metadata_through_facade() {
    let dir = mktemp_dir("generic_function_facade");
    std::fs::write(
        dir.join("helper.sifr"),
        "def identity[T](value: T) -> T:\n    return value\n",
    )
    .expect("helper should be written");
    std::fs::write(
        dir.join("facade.sifr"),
        "from helper import identity as public_identity\n",
    )
    .expect("facade should be written");
    std::fs::write(
        dir.join("main.sifr"),
        "from facade import public_identity as identity\n\ndef main():\n    value: int = identity(1)\n    assert value == 1\n",
    )
    .expect("main should be written");

    let errors = check_project(&dir.join("main.sifr"));
    assert!(errors.is_empty(), "generic import should check: {errors:?}");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_build_project_imports_generic_function_metadata_through_facade() {
    let dir = mktemp_dir("generic_function_facade_native");
    std::fs::write(
        dir.join("helper.sifr"),
        "def identity[T](value: T) -> T:\n    return value\n",
    )
    .expect("helper should be written");
    std::fs::write(
        dir.join("facade.sifr"),
        "from helper import identity as public_identity\n",
    )
    .expect("facade should be written");
    std::fs::write(
        dir.join("main.sifr"),
        "from facade import public_identity as identity\n\ndef main():\n    value: int = identity(1)\n    assert value == 1\n",
    )
    .expect("main should be written");
    let binary = build_project(&dir.join("main.sifr"), &dir.join("build_out"))
        .expect("generic function facade should build natively");
    assert!(binary.exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_check_project_preserves_imported_generic_function_bounds() {
    let dir = mktemp_dir("generic_function_bounds");
    std::fs::write(
        dir.join("helper.sifr"),
        "def identity[T: bigint](value: T) -> T:\n    return value\n",
    )
    .expect("helper should be written");
    std::fs::write(
        dir.join("facade.sifr"),
        "from helper import identity as public_identity\n",
    )
    .expect("facade should be written");
    std::fs::write(
        dir.join("main.sifr"),
        "from facade import public_identity as identity\n\ndef main():\n    identity(\"wrong\")\n",
    )
    .expect("main should be written");

    let errors = check_project(&dir.join("main.sifr"));
    assert!(
        errors.iter().any(|error| {
            error.message.contains("does not satisfy bound")
                || error.message.contains("does not satisfy constraint")
                || error
                    .message
                    .contains("does not implement protocol 'bigint'")
        }),
        "generic import bounds should reject invalid specialization: {errors:?}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

fn write_split_ancestry_project(dir: &std::path::Path) {
    std::fs::write(dir.join("base.sifr"), "class Root:\n    value: int\n")
        .expect("base should be written");
    std::fs::write(
        dir.join("child.sifr"),
        "from base import Root as R\n\nclass Child(R):\n    extra: int\n\n    def __init__(self, value: int, extra: int):\n        super().__init__(value)\n        self.extra = extra\n",
    )
    .expect("child should be written");
    std::fs::write(
        dir.join("roots.sifr"),
        "from base import Root as PublicRoot\n",
    )
    .expect("roots facade should be written");
    std::fs::write(
        dir.join("children.sifr"),
        "from child import Child as PublicChild\n",
    )
    .expect("children facade should be written");
    std::fs::write(
        dir.join("main.sifr"),
        "from roots import PublicRoot as Root\nfrom children import PublicChild as Child\n\ndef accept(value: Root) -> int:\n    return value.value\n\ndef main():\n    child: Child = Child(1, 2)\n    assert accept(child) == 1\n",
    )
    .expect("main should be written");
}

#[test]
fn test_check_project_canonicalizes_imported_parent_ancestry() {
    let dir = mktemp_dir("imported_parent_ancestry");
    write_split_ancestry_project(&dir);
    let errors = check_project(&dir.join("main.sifr"));
    assert!(errors.is_empty(), "split ancestry should check: {errors:?}");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_build_project_canonicalizes_imported_parent_ancestry() {
    let dir = mktemp_dir("imported_parent_ancestry_native");
    write_split_ancestry_project(&dir);
    let binary = build_project(&dir.join("main.sifr"), &dir.join("build_out"))
        .expect("split imported ancestry should build natively");
    assert!(binary.exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_build_project_specializes_inferred_generic_returns() {
    let dir = mktemp_dir("inferred_generic_return_native");
    std::fs::write(
        dir.join("main.sifr"),
        "class Box[T]:\n    value: T\n\ndef identity[T](value: T) -> T:\n    return value\n\ndef make_box():\n    return Box(1)\n\ndef make_value():\n    return identity(2)\n\ndef main():\n    value: Box[int] = make_box()\n    assert value.value + make_value() == 3\n",
    )
    .expect("main should be written");
    let binary = build_project(&dir.join("main.sifr"), &dir.join("build_out"))
        .expect("inferred generic returns should build natively");
    assert!(binary.exists());
    let _ = std::fs::remove_dir_all(dir);
}
