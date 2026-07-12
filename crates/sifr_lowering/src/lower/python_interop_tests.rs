use crate::{lower_module, HirDiagnostic, HirExpr, HirModule, HirStmt};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::{PythonInteropEffect, PythonParameterKind};
use sifr_python_parser::parse_module;

fn lower_ok(source: &str) -> HirModule {
    let parsed = parse_module(source).expect("source should parse");
    lower_module(parsed.suite())
        .map(|result| result.module)
        .expect("source should lower")
}

fn lower_errors(source: &str) -> Vec<HirDiagnostic> {
    let parsed = parse_module(source).expect("source should parse");
    match lower_module(parsed.suite()) {
        Ok(_) => panic!("source should fail lowering"),
        Err(errors) => errors,
    }
}

#[test]
fn sync_python_declaration_retains_target_effect_and_call_shape() {
    let module = lower_ok(
        r#"
class PythonError(Error):
    message: str

@python(pkg.api.compute)
def compute(value: int, *rest: int, flag: bool = False, missing: int = python.omit, **extra: int) -> Result[int, PythonError]: ...
"#,
    );
    let function = &module.functions[0];
    let declaration = &function.python_interop[0];
    assert!(function.body.is_empty());
    assert_eq!(declaration.effect, PythonInteropEffect::BlockingIo);
    assert_eq!(declaration.required_import_root.as_deref(), Some("pkg"));
    assert_eq!(
        declaration.target.as_ref().expect("target").dotted(),
        "pkg.api.compute"
    );
    assert_eq!(
        declaration
            .parameters
            .iter()
            .map(|parameter| parameter.kind)
            .collect::<Vec<_>>(),
        vec![
            PythonParameterKind::Positional,
            PythonParameterKind::PositionalVariadic,
            PythonParameterKind::KeywordOnly,
            PythonParameterKind::KeywordOnly,
            PythonParameterKind::KeywordVariadic,
        ]
    );
    assert!(declaration.parameters[3].omit_when_absent);
    assert_eq!(function.params.len(), 5);
}

#[test]
fn invalid_python_target_reports_pyimp_0001() {
    let errors = lower_errors(
        r#"
class PythonError(Error):
    message: str

@python("pkg.compute")
def compute(value: int) -> Result[int, PythonError]: ...
"#,
    );
    assert!(errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYIMP_INVALID_TARGET)));
}

#[test]
fn invalid_python_declaration_shape_reports_pycall_0001() {
    let errors = lower_errors(
        r"
class PythonError(Error):
    message: str

@python(pkg.compute, pkg.other)
def compute(value: int) -> Result[int, PythonError]: ...
",
    );
    assert!(errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYCALL_INVALID_SHAPE)));
}

#[test]
fn unsupported_python_conversion_reports_pyconv_0001() {
    let errors = lower_errors(
        r"
class PythonError(Error):
    message: str

@python(pkg.compute)
def compute(values: list[int]) -> Result[int, PythonError]: ...
",
    );
    assert!(errors
        .iter()
        .any(|error| { error.code == Some(DiagnosticCode::PYCONV_UNSUPPORTED_DECLARATION_TYPE) }));
}

#[test]
fn later_python_decorator_is_a_hard_error() {
    let errors = lower_errors(
        r"
class PythonError(Error):
    message: str

@python.coroutine(pkg.compute)
async def compute(value: int) -> Result[int, PythonError]: ...
",
    );
    assert!(errors
        .iter()
        .any(|error| { error.code == Some(DiagnosticCode::PYRES_UNIMPLEMENTED_DECLARATION) }));
}

#[test]
fn python_declaration_rejects_non_stub_body() {
    let errors = lower_errors(
        r"
class PythonError(Error):
    message: str

@python(pkg.compute)
def compute(value: int) -> Result[int, PythonError]:
    ...
    return Ok(value)
",
    );
    assert!(errors
        .iter()
        .any(|error| error.code == Some(DiagnosticCode::PYCALL_INVALID_SHAPE)));
}

#[test]
fn sync_python_declaration_is_blocking_in_async_context() {
    let errors = lower_errors(
        r"
class PythonError(Error):
    message: str

@python(math.sqrt)
def sqrt(value: float) -> Result[float, PythonError]: ...

async def use_sqrt():
    sqrt(4.0)
    await sleep(0.0)
",
    );
    assert!(errors
        .iter()
        .any(|error| { error.code == Some(DiagnosticCode::ASYNC_DIRECT_BLOCKING_IO_CALL) }));
}

#[test]
fn python_calls_lower_omit_none_variadics_and_typed_kwargs() {
    let module = lower_ok(
        r#"
class PythonError(Error):
    message: str

@python(pkg.collect)
def collect(value: int, *rest: int, label: str | None = python.omit, **extra: int) -> Result[int, PythonError]: ...

def main():
    first = collect(1)
    second = collect(1, 2, 3, label=None, count=4)
    options: dict[str, int] = {"count": 5}
    third = collect(1, **options)
    _ = first
    _ = second
    _ = third
"#,
    );
    let declaration = &module.functions[0];
    assert_eq!(declaration.params.len(), 4);
    assert_eq!(declaration.python_interop[0].parameters.len(), 4);
    assert!(declaration.python_interop[0].parameters[2].omit_when_absent);
}

#[test]
fn closed_record_expansion_retains_fields_and_span_in_hir() {
    let module = lower_ok(
        r#"
class PythonError(Error):
    message: str

class Options:
    label: str
    count: int

    def __init__(self, label: str, count: int):
        self.label = label
        self.count = count

@python(pkg.collect)
def collect(*, label: str, count: int) -> Result[int, PythonError]: ...

def main():
    options = Options("x", 2)
    result = collect(**options)
    _ = result
"#,
    );
    let main = module
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main");
    let call = main.body.iter().find_map(|statement| match statement {
        HirStmt::Let {
            value: call @ HirExpr::PythonCall { .. },
            ..
        } => Some(call),
        _ => None,
    });
    let Some(HirExpr::PythonCall {
        record_expansions, ..
    }) = call
    else {
        panic!("record call should retain PythonCall metadata");
    };
    assert_eq!(record_expansions.len(), 1);
    assert_eq!(record_expansions[0].fields, ["label", "count"]);
    assert!(!record_expansions[0].span.is_empty());
}
