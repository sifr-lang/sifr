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
        "from roots import PublicRoot as Root\nfrom children import PublicChild as Child\n\ndef accept(value: Root) -> int:\n    return value.value\n\ndef consume(own value: Root) -> int:\n    return value.value\n\ndef as_root(own value: Child) -> Root:\n    return value\n\ndef as_union(own value: Child) -> Root | int:\n    return value\n\ndef as_result(own value: Child) -> Result[Root, ValueError]:\n    return value\n\ndef main():\n    borrowed: Child = Child(1, 2)\n    assert accept(borrowed) == 1\n    owned: Child = Child(3, 4)\n    assert consume(owned) == 3\n    root: Root = as_root(Child(5, 6))\n    assert root.value == 5\n    union_value: Root | int = as_union(Child(7, 8))\n    result_value: Result[Root, ValueError] = as_result(Child(9, 10))\n",
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
    let status = std::process::Command::new(&binary)
        .status()
        .expect("split imported ancestry binary should run");
    assert!(status.success());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_build_project_specializes_zero_argument_generic_return() {
    let dir = mktemp_dir("zero_argument_generic_return_native");
    std::fs::write(
        dir.join("main.sifr"),
        "class Marker[T]:\n    pass\n\nclass Local(NonSend):\n    pass\n\ndef make[T]() -> Marker[T]:\n    return Marker()\n\ndef relay(own marker: Marker[Local]) -> Marker[Local]:\n    return marker\n\ndef main():\n    inferred: Marker[int] = make()\n    marker: Marker[Local] = Marker()\n    moved: Marker[Local] = relay(marker)\n",
    )
    .expect("main should be written");
    let binary = build_project(&dir.join("main.sifr"), &dir.join("build_out"))
        .expect("zero-argument generic return should build natively");
    let status = std::process::Command::new(&binary)
        .status()
        .expect("zero-argument generic return binary should run");
    assert!(status.success());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_build_project_consumes_transitive_class_upcasts() {
    let dir = mktemp_dir("consuming_class_upcast_native");
    std::fs::write(
        dir.join("main.sifr"),
        "class Root:\n    value: int\n\nclass Mid(Root):\n    middle: int\n\n    def __init__(self, value: int, middle: int):\n        super().__init__(value)\n        self.middle = middle\n\nclass Child(Mid):\n    extra: int\n\n    def __init__(self, value: int, middle: int, extra: int):\n        super().__init__(value, middle)\n        self.extra = extra\n\ndef consume(own value: Root) -> int:\n    return value.value\n\ndef as_root(own value: Child) -> Root:\n    return value\n\ndef as_union(own value: Child) -> Root | int:\n    return value\n\ndef as_result(own value: Child) -> Result[Root, ValueError]:\n    return value\n\ndef main():\n    child: Child = Child(1, 2, 3)\n    assert consume(child) == 1\n    root: Root = as_root(Child(4, 5, 6))\n    assert root.value == 4\n    union_value: Root | int = as_union(Child(7, 8, 9))\n    result_value: Result[Root, ValueError] = as_result(Child(10, 11, 12))\n",
    )
    .expect("main should be written");
    let binary = build_project(&dir.join("main.sifr"), &dir.join("build_out"))
        .expect("consuming class upcasts should build natively");
    let status = std::process::Command::new(&binary)
        .status()
        .expect("consuming class upcast binary should run");
    assert!(status.success());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_build_project_prefers_exact_same_basename_ancestor() {
    let dir = mktemp_dir("same_basename_ancestor_native");
    std::fs::write(dir.join("base.sifr"), "class Root:\n    value: int\n")
        .expect("base should be written");
    std::fs::write(
        dir.join("middle.sifr"),
        "from base import Root as BaseRoot\n\nclass Root(BaseRoot):\n    middle: int\n\n    def __init__(self, value: int, middle: int):\n        super().__init__(value)\n        self.middle = middle\n",
    )
    .expect("middle should be written");
    std::fs::write(
        dir.join("child.sifr"),
        "from middle import Root as MiddleRoot\n\nclass Child(MiddleRoot):\n    extra: int\n\n    def __init__(self, value: int, middle: int, extra: int):\n        super().__init__(value, middle)\n        self.extra = extra\n",
    )
    .expect("child should be written");
    std::fs::write(
        dir.join("facade.sifr"),
        "from base import Root as PublicRoot\nfrom child import Child as PublicChild\n",
    )
    .expect("facade should be written");
    std::fs::write(
        dir.join("main.sifr"),
        "from facade import PublicRoot as Root, PublicChild as Child\n\ndef as_root(own value: Child) -> Root:\n    return value\n\ndef main():\n    root: Root = as_root(Child(1, 2, 3))\n    assert root.value == 1\n",
    )
    .expect("main should be written");
    let binary = build_project(&dir.join("main.sifr"), &dir.join("build_out"))
        .expect("same-basename ancestry should build natively");
    let status = std::process::Command::new(&binary)
        .status()
        .expect("same-basename ancestry binary should run");
    assert!(status.success());
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
