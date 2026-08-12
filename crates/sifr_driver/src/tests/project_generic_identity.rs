use super::project_build_check::mktemp_dir;
use crate::{build, build_project, check_project};

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
fn test_build_project_shares_union_identity_across_modules() {
    let dir = mktemp_dir("shared_project_union_identity");
    std::fs::write(
        dir.join("errors.sifr"),
        "class FirstError(Error):\n    message: str\n\nclass SecondError(Error):\n    message: str\n\nclass Payload:\n    value: bool | int | float | str | bytes | None\n    tag: int | str\n\ndef produce() -> Result[int, FirstError | SecondError]:\n    return 7\n\ndef make_payload() -> Payload:\n    return Payload(True, 3)\n",
    )
    .expect("union provider should be written");
    std::fs::write(
        dir.join("facade.sifr"),
        "from errors import FirstError as PublicFirstError, Payload as PublicPayload, SecondError as PublicSecondError, make_payload as public_make_payload, produce as public_produce\n",
    )
    .expect("union facade should be written");
    std::fs::write(
        dir.join("main.sifr"),
        "from facade import PublicFirstError as E1, PublicPayload as Payload, PublicSecondError as E2, public_make_payload as make_payload, public_produce as produce\n\ndef relay() -> Result[int, E1 | E2]:\n    result: Result[int, E1 | E2] = produce()\n    return result\n\ndef tag_value(payload: Payload) -> int:\n    tag: int | str = payload.tag\n    if isinstance(tag, int):\n        return tag\n    else:\n        return len(tag)\n\ndef main():\n    payload: Payload = make_payload()\n    assert payload.value == True\n    assert tag_value(payload) == 3\n",
    )
    .expect("union consumer should be written");

    let binary = build_project(&dir.join("main.sifr"), &dir.join("build_out"))
        .expect("one project-wide union identity should build natively");
    assert!(binary.exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_build_project_centralizes_stdlib_nominal_union_payload() {
    let dir = mktemp_dir("shared_stdlib_nominal_union");
    std::fs::write(
        dir.join("helper.sifr"),
        "from sifr.pathlib import Path\n\ndef pick(flag: bool) -> Path | int:\n    if flag:\n        return Path(\"/tmp\")\n    return 1\n\ndef go() -> int:\n    value: Path | int = pick(True)\n    return 1\n",
    )
    .expect("helper should be written");
    std::fs::write(
        dir.join("main.sifr"),
        "from helper import go\nfrom sifr.pathlib import Path\n\ndef main():\n    local: Path | int = Path(\"/var\")\n    assert go() == 1\n",
    )
    .expect("main should be written");

    let binary = build_project(&dir.join("main.sifr"), &dir.join("build_out"))
        .expect("stdlib nominal union payload should have one project type");
    let status = std::process::Command::new(&binary)
        .status()
        .expect("generated binary should run");
    assert!(status.success());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_build_project_isolates_union_prelude_imports() {
    let dir = mktemp_dir("isolated_union_prelude_imports");
    std::fs::write(
        dir.join("helper.sifr"),
        "def pick(flag: bool) -> dict[str, int] | int:\n    if flag:\n        return {\"value\": 1}\n    return 2\n",
    )
    .expect("helper should be written");
    std::fs::write(
        dir.join("main.sifr"),
        "from helper import pick\n\ndef main():\n    local: dict[str, int] = {\"value\": 3}\n    selected: dict[str, int] | int = pick(True)\n    assert local[\"value\"] == 3\n",
    )
    .expect("main should be written");

    let binary = build_project(&dir.join("main.sifr"), &dir.join("build_out"))
        .expect("union prelude imports should not conflict with root imports");
    let status = std::process::Command::new(&binary)
        .status()
        .expect("generated binary should run");
    assert!(status.success());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_build_project_keeps_same_basename_enum_and_newtype_unions_distinct() {
    let dir = mktemp_dir("distinct_enum_newtype_union_identities");
    let provider = |status_value: i64, token_value: i64| {
        format!(
            "from enum import Enum\n\nclass Status(Enum):\n    READY = {status_value}\n\nclass Token(int):\n    pass\n\ndef status() -> Status | int:\n    return Status.READY\n\ndef token() -> Token | str:\n    return Token({token_value})\n"
        )
    };
    std::fs::write(dir.join("left.sifr"), provider(1, 11))
        .expect("left provider should be written");
    std::fs::write(dir.join("right.sifr"), provider(2, 22))
        .expect("right provider should be written");
    std::fs::write(
        dir.join("main.sifr"),
        "from left import Status as LeftStatus, Token as LeftToken, status as left_status, token as left_token\nfrom right import Status as RightStatus, Token as RightToken, status as right_status, token as right_token\n\ndef main():\n    first_status: LeftStatus | int = left_status()\n    second_status: RightStatus | int = right_status()\n    first_token: LeftToken | str = left_token()\n    second_token: RightToken | str = right_token()\n",
    )
    .expect("union consumer should be written");

    let binary = build_project(&dir.join("main.sifr"), &dir.join("build_out"))
        .expect("same-basename enum and newtype unions should remain distinct");
    assert!(binary.exists());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_check_project_keeps_same_basename_protocol_unions_distinct() {
    let dir = mktemp_dir("distinct_protocol_union_identities");
    let provider = "class Readable(Protocol):\n    def read(self) -> str:\n        pass\n\ndef accept(value: Readable | int) -> int:\n    return 1\n";
    std::fs::write(dir.join("left.sifr"), provider).expect("left provider should be written");
    std::fs::write(dir.join("right.sifr"), provider).expect("right provider should be written");
    std::fs::write(
        dir.join("main.sifr"),
        "from left import Readable as LeftReadable, accept as accept_left\nfrom right import Readable as RightReadable, accept as accept_right\n\ndef use_left(value: LeftReadable | int) -> int:\n    return accept_left(value)\n\ndef use_right(value: RightReadable | int) -> int:\n    return accept_right(value)\n\ndef main():\n    pass\n",
    )
    .expect("protocol consumer should be written");

    let errors = check_project(&dir.join("main.sifr"));
    assert!(
        errors.is_empty(),
        "same-basename protocol unions should remain distinct: {errors:?}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_build_single_file_consumes_three_level_class_upcast() {
    let dir = mktemp_dir("single_file_three_level_upcast");
    let source = "class Root:\n    value: int\n\nclass Mid(Root):\n    middle: int\n\n    def __init__(self, value: int, middle: int):\n        super().__init__(value)\n        self.middle = middle\n\nclass Child(Mid):\n    extra: int\n\n    def __init__(self, value: int, middle: int, extra: int):\n        super().__init__(value, middle)\n        self.extra = extra\n\ndef consume(own value: Root) -> int:\n    return value.value\n\ndef as_root(own value: Child) -> Root:\n    return value\n\ndef main():\n    assert consume(Child(1, 2, 3)) == 1\n    root: Root = as_root(Child(4, 5, 6))\n    assert root.value == 4\n";

    let binary = build(source, &dir.join("build_out"))
        .expect("single-file transitive class upcast should build natively");
    let status = std::process::Command::new(&binary)
        .status()
        .expect("single-file transitive class upcast binary should run");
    assert!(status.success());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_build_project_keeps_same_basename_union_identities_distinct() {
    let dir = mktemp_dir("distinct_project_union_identities");
    let provider = |value: i64| {
        format!(
            "class PackageError(Error):\n    message: str\n\nclass OtherError(Error):\n    message: str\n\ndef produce() -> Result[int, PackageError | OtherError]:\n    return {value}\n"
        )
    };
    std::fs::write(dir.join("left.sifr"), provider(1)).expect("left provider should be written");
    std::fs::write(dir.join("right.sifr"), provider(2)).expect("right provider should be written");
    std::fs::write(
        dir.join("main.sifr"),
        "from left import PackageError as LeftPackageError, OtherError as LeftOtherError, produce as produce_left\nfrom right import PackageError as RightPackageError, OtherError as RightOtherError, produce as produce_right\n\ndef left_value() -> Result[int, LeftPackageError | LeftOtherError]:\n    return produce_left()\n\ndef right_value() -> Result[int, RightPackageError | RightOtherError]:\n    return produce_right()\n\ndef main():\n    pass\n",
    )
    .expect("union consumer should be written");

    let binary = build_project(&dir.join("main.sifr"), &dir.join("build_out"))
        .expect("same-basename unions should remain nominally distinct");
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

#[test]
fn test_check_project_preserves_same_basename_affine_capabilities() {
    let dir = mktemp_dir("same_basename_affine_capability");
    std::fs::write(
        dir.join("inner.sifr"),
        "class Root:\n    view: python.Buffer[uint8]\n",
    )
    .expect("inner class should be written");
    std::fs::write(
        dir.join("outer.sifr"),
        "from inner import Root as InnerRoot\n\nclass Root:\n    inner: InnerRoot\n",
    )
    .expect("outer class should be written");
    std::fs::write(
        dir.join("main.sifr"),
        "from outer import Root\n\ndef duplicate(own value: Root) -> None:\n    first = second = value\n",
    )
    .expect("main should be written");

    let errors = check_project(&dir.join("main.sifr"));
    assert!(
        errors.iter().any(|error| {
            error.message.contains("affine")
                || error.message.contains("cannot be cloned")
                || error.message.contains("cannot be duplicated")
        }),
        "same-basename nested buffer should remain affine: {errors:?}"
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
        "from roots import PublicRoot as Root\nfrom children import PublicChild as Child\n\ndef accept(value: Root) -> int:\n    return value.value\n\ndef consume(own value: Root) -> int:\n    return value.value\n\ndef consume_union(own value: Root | int) -> int:\n    return 11\n\ndef consume_result(own value: Result[Root, ValueError]) -> int:\n    return 12\n\ndef as_root(own value: Child) -> Root:\n    return value\n\ndef as_union(own value: Child) -> Root | int:\n    return value\n\ndef as_result(own value: Child) -> Result[Root, ValueError]:\n    return value\n\ndef child_result() -> Result[Child, ValueError]:\n    return Child(15, 16)\n\ndef root_result() -> Result[Root, ValueError]:\n    return child_result()\n\ndef main():\n    borrowed: Child = Child(1, 2)\n    assert accept(borrowed) == 1\n    owned: Child = Child(3, 4)\n    assert consume(owned) == 3\n    root: Root = as_root(Child(5, 6))\n    assert root.value == 5\n    union_value: Root | int = as_union(Child(7, 8))\n    result_value: Result[Root, ValueError] = as_result(Child(9, 10))\n    assert consume_union(Child(17, 18)) == 11\n    assert consume_result(child_result()) == 12\n    remapped_result: Result[Root, ValueError] = root_result()\n",
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
fn test_build_project_crate_roots_non_main_transitive_upcasts() {
    let dir = mktemp_dir("non_main_transitive_upcast_native");
    std::fs::write(
        dir.join("shapes.sifr"),
        "class Root:\n    value: int\n\nclass Mid(Root):\n    middle: int\n\n    def __init__(self, value: int, middle: int):\n        super().__init__(value)\n        self.middle = middle\n\nclass Child(Mid):\n    extra: int\n\n    def __init__(self, value: int, middle: int, extra: int):\n        super().__init__(value, middle)\n        self.extra = extra\n",
    )
    .expect("shape declarations should be written");
    std::fs::write(
        dir.join("adapter.sifr"),
        "from shapes import Child, Root\n\ndef as_root(own value: Child) -> Root:\n    return value\n",
    )
    .expect("non-main adapter should be written");
    std::fs::write(
        dir.join("main.sifr"),
        "from adapter import as_root\nfrom shapes import Child, Root\n\ndef main():\n    root: Root = as_root(Child(1, 2, 3))\n    assert root.value == 1\n",
    )
    .expect("main consumer should be written");

    let binary = build_project(&dir.join("main.sifr"), &dir.join("build_out"))
        .expect("non-main transitive upcasts should use crate-rooted paths");
    let status = std::process::Command::new(&binary)
        .status()
        .expect("non-main transitive upcast binary should run");
    assert!(status.success());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
#[ignore = "generated build integration coverage runs in full validation profiles"]
fn test_build_project_remaps_structural_consuming_upcasts() {
    let dir = mktemp_dir("structural_consuming_upcast_native");
    std::fs::write(
        dir.join("main.sifr"),
        "class Root:\n    value: int\n\nclass Mid(Root):\n    middle: int\n\n    def __init__(self, value: int, middle: int):\n        super().__init__(value)\n        self.middle = middle\n\nclass Child(Mid):\n    extra: int\n\n    def __init__(self, value: int, middle: int, extra: int):\n        super().__init__(value, middle)\n        self.extra = extra\n\ndef make_union() -> Child | int:\n    return Child(1, 2, 3)\n\ndef relay_union() -> Root | int:\n    return make_union()\n\ndef consume_union(own value: Root | int) -> int:\n    return 21\n\ndef borrow_union(value: Root | int) -> int:\n    return 23\n\ndef make_option() -> Child | None:\n    return Child(10, 11, 12)\n\ndef relay_option() -> Root | None:\n    return make_option()\n\ndef consume_option(own value: Root | None) -> int:\n    return 24\n\ndef make_result() -> Result[Child, ValueError]:\n    return Child(4, 5, 6)\n\ndef relay_result() -> Result[Root, ValueError]:\n    return make_result()\n\ndef consume_result(own value: Result[Root, ValueError]) -> int:\n    return 22\n\ndef main():\n    assert consume_union(Child(7, 8, 9)) == 21\n    assert consume_union(make_union()) == 21\n    borrowed: Child | int = make_union()\n    assert borrow_union(borrowed) == 23\n    assert consume_option(make_option()) == 24\n    assert consume_result(make_result()) == 22\n    union_value: Root | int = relay_union()\n    option_value: Root | None = relay_option()\n    result_value: Result[Root, ValueError] = relay_result()\n",
    )
    .expect("main should be written");
    let binary = build_project(&dir.join("main.sifr"), &dir.join("build_out"))
        .expect("structural consuming upcasts should build natively");
    let status = std::process::Command::new(&binary)
        .status()
        .expect("structural consuming upcast binary should run");
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
