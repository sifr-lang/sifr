use crate::{collect_module_exports, compile_module_hir, FrontendDiagnosticStyle};
use sifr_lowering::{ExternalDefs, LoweringResult, StaticMethodSlotContext};
use sifr_syntax::parse_module_suite;

fn compile(
    module: &str,
    source: &str,
    external_defs: &ExternalDefs,
) -> Result<LoweringResult, Vec<sifr_diagnostics::RenderedDiagnostic>> {
    let parsed = parse_module_suite(source, None).expect("fixture parses");
    compile_module_hir(
        module,
        &parsed,
        external_defs,
        FrontendDiagnosticStyle::Bare,
    )
}

fn install_specializer(external_defs: &mut ExternalDefs, slot_references: &[&str]) {
    let slots = slot_references
        .iter()
        .map(|reference| format!("\"{reference}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!(
        r#"
class Outcome:
    status: str
    value: dict[str, list[str]] | None
    issues: list[str]

@const_eval
def describe(shape: dict[str, str]) -> Outcome:
    return Outcome("produced", {{"sifr.meta.slots": [{slots}]}}, [])
"#
    );
    let package =
        compile("fixture.slots", &source, external_defs).expect("slot specializer compiles");
    collect_module_exports("fixture.slots", &package, external_defs);
}

fn diagnostic_codes(
    result: Result<LoweringResult, Vec<sifr_diagnostics::RenderedDiagnostic>>,
) -> Vec<String> {
    match result {
        Ok(_) => panic!("fixture must fail"),
        Err(diagnostics) => diagnostics
            .into_iter()
            .map(|diagnostic| diagnostic.code)
            .collect(),
    }
}

#[test]
fn reexported_method_slot_resolves_by_qualified_owner_identity() {
    let mut external_defs = ExternalDefs::default();
    install_specializer(&mut external_defs, &["models.Record::normalize"]);
    let models = compile(
        "models",
        r#"
class Record:
    value: str

    @staticmethod
    @metadata("fixture.slot", "normalize")
    def normalize(own value: str) -> Result[str, ValueError]:
        return value
"#,
        &external_defs,
    )
    .expect("model module compiles");
    collect_module_exports("models", &models, &mut external_defs);
    let facade = compile(
        "facade",
        "from models import Record as PublicRecord\n",
        &external_defs,
    )
    .expect("facade compiles");
    collect_module_exports("facade", &facade, &mut external_defs);

    let consumer = compile(
        "consumer",
        r#"
from facade import PublicRecord
from fixture.slots import describe

@const_specialize("fixture.slots", "describe")
class Container:
    item: PublicRecord
"#,
        &external_defs,
    )
    .expect("re-exported slot owner specializes");
    let output = &consumer.specialization_outputs[0];
    assert_eq!(output.method_slots[0].owner_identity, "models.Record");
    assert_eq!(
        output.method_slot_context,
        Some(StaticMethodSlotContext::None)
    );
}

#[test]
fn unavailable_slot_target_uses_method_diagnostic() {
    let mut external_defs = ExternalDefs::default();
    install_specializer(&mut external_defs, &["target.Record::missing"]);
    let codes = diagnostic_codes(compile(
        "target",
        r#"
from fixture.slots import describe

@const_specialize("fixture.slots", "describe")
class Record:
    value: str
"#,
        &external_defs,
    ));
    assert!(codes.iter().any(|code| code == "SIFR-RUST-SLOT-0002"));
}

#[test]
fn non_result_slot_uses_signature_diagnostic() {
    let mut external_defs = ExternalDefs::default();
    install_specializer(&mut external_defs, &["target.Record::normalize"]);
    let codes = diagnostic_codes(compile(
        "target",
        r#"
from fixture.slots import describe

@const_specialize("fixture.slots", "describe")
class Record:
    value: str

    @staticmethod
    @metadata("fixture.slot", "normalize")
    def normalize(own value: str) -> str:
        return value
"#,
        &external_defs,
    ));
    assert!(codes.iter().any(|code| code == "SIFR-RUST-SLOT-0003"));
}

#[test]
fn conflicting_context_borrow_modes_use_context_diagnostic() {
    let mut external_defs = ExternalDefs::default();
    install_specializer(
        &mut external_defs,
        &["target.Record::before", "target.Record::after"],
    );
    let codes = diagnostic_codes(compile(
        "target",
        r#"
from fixture.slots import describe

class AppContext:
    calls: int

@const_specialize("fixture.slots", "describe")
class Record:
    value: str

    @staticmethod
    @metadata("fixture.slot", "before")
    def before(own value: str, context: AppContext) -> Result[str, ValueError]:
        return value

    @staticmethod
    @metadata("fixture.slot", "after")
    def after(own value: str, mut context: AppContext) -> Result[str, ValueError]:
        context.calls += 1
        return value
"#,
        &external_defs,
    ));
    assert!(codes.iter().any(|code| code == "SIFR-RUST-SLOT-0005"));
}
