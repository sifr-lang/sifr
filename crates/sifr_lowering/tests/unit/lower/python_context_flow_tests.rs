use super::python_interop_tests::lower_ok;
use crate::{HirStmt, hir_nodes::HirWithItemKind};

#[test]
fn python_with_infallible_return_body_remains_terminal() {
    let source = r#"
class PythonError(Error):
    message: str
    kind: str
    exception_type: str
    traceback: str
    context: str

class ExitCause:
    pass

class ExitDecision:
    pass

@python.opaque(type=pkg.Counter, cleanup=context)
class Counter:
    @python.context.enter(Self.__enter__)
    def __enter__(self) -> Result[int, PythonError]: ...

    @python.context.exit(Self.__exit__)
    def __exit__(own self, cause: ExitCause) -> Result[ExitDecision, PythonError]: ...

@python(pkg.Counter)
def make_counter() -> Result[Counter, PythonError]: ...

def use_counter() -> Result[int, PythonError]:
    try:
        with make_counter() as value:
            copied: int = value
            return copied
    except PythonError as error:
        raise error
"#;
    let module = lower_ok(source);
    let function = module
        .functions
        .iter()
        .find(|function| function.name == "use_counter")
        .expect("function");
    let HirStmt::TryExcept { body, .. } = &function.body[0] else {
        panic!("function should retain try body");
    };
    let HirStmt::With { items, .. } = &body[0] else {
        panic!("body should retain with statement");
    };
    assert!(matches!(
        items[0].kind,
        HirWithItemKind::Python {
            entered_is_opaque_borrow: false,
            body_may_raise: false,
            ..
        }
    ));
}
