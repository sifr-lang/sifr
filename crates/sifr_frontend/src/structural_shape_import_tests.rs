use crate::{
    collect_module_exports, compile_module_hir, describe_type_with_externals,
    FrontendDiagnosticStyle,
};
use sifr_lowering::{ExternalDefs, LoweringResult};
use sifr_syntax::parse_module_suite;
use sifr_type_system::Type;

fn compile(module: &str, source: &str, external_defs: &ExternalDefs) -> LoweringResult {
    let parsed = parse_module_suite(source, None).expect("fixture parses");
    compile_module_hir(
        module,
        &parsed,
        external_defs,
        FrontendDiagnosticStyle::Bare,
    )
    .expect("fixture compiles")
}

fn field_type<'a>(result: &'a LoweringResult, class_name: &str, field: &str) -> &'a Type {
    result
        .module
        .classes
        .iter()
        .find(|class| class.name == class_name)
        .and_then(|class| class.fields.iter().find(|(name, _)| name == field))
        .map(|(_, ty)| ty)
        .expect("field exists")
}

#[test]
fn recollection_removes_stale_structural_shape_exports() {
    let mut external_defs = ExternalDefs::default();
    let annotated = compile(
        "models",
        r#"
@metadata("fixture.kind", "hidden")
class _Hidden[T]:
    value: T = 0

    @metadata("fixture.callback", "read")
    def read(self) -> T:
        return self.value
"#,
        &external_defs,
    );
    collect_module_exports("models", &annotated, &mut external_defs);
    assert!(external_defs.class_field_defaults.contains_key("models"));
    assert!(external_defs.declaration_metadata.contains_key("models"));
    assert!(external_defs.class_type_params.contains_key("models"));
    assert!(external_defs.structural_methods_for("models").is_some());

    let plain = compile("models", "class _Hidden:\n    value: int\n", &external_defs);
    collect_module_exports("models", &plain, &mut external_defs);
    assert!(!external_defs.class_field_defaults.contains_key("models"));
    assert!(!external_defs.declaration_metadata.contains_key("models"));
    assert!(!external_defs.class_type_params.contains_key("models"));
    assert!(external_defs.structural_methods_for("models").is_none());
}

#[test]
fn structural_method_storage_is_allocated_only_while_demanded() {
    let mut external_defs = ExternalDefs::default();
    let plain = compile("first", "class Plain:\n    value: int\n", &external_defs);
    collect_module_exports("first", &plain, &mut external_defs);
    assert!(!external_defs.has_structural_methods());

    let annotated_source = r#"
class Model:
    @metadata("fixture.callback", "read")
    def read(self) -> int:
        return 1
"#;
    let first = compile("first", annotated_source, &external_defs);
    collect_module_exports("first", &first, &mut external_defs);
    assert!(external_defs.has_structural_methods());

    let first_plain = compile("first", "class Model:\n    pass\n", &external_defs);
    collect_module_exports("first", &first_plain, &mut external_defs);
    assert!(!external_defs.has_structural_methods());

    let first = compile("first", annotated_source, &external_defs);
    let second = compile("second", annotated_source, &external_defs);
    collect_module_exports("first", &first, &mut external_defs);
    collect_module_exports("second", &second, &mut external_defs);
    collect_module_exports("first", &first_plain, &mut external_defs);
    assert!(external_defs.has_structural_methods());
    let second_plain = compile("second", "class Model:\n    pass\n", &external_defs);
    collect_module_exports("second", &second_plain, &mut external_defs);
    assert!(!external_defs.has_structural_methods());
}

#[test]
fn imported_nominal_ignores_colliding_local_class_shape() {
    let mut external_defs = ExternalDefs::default();
    let models = compile(
        "models",
        r#"
class Box:
    model_value: int = 1

    @metadata("fixture.callback", "model")
    def model_method(self) -> int:
        return self.model_value

class Wrapper:
    item: Box

class LocalUse:
    value: Wrapper
"#,
        &external_defs,
    );
    let local = describe_type_with_externals(
        "models",
        field_type(&models, "LocalUse", "value"),
        &models,
        &external_defs,
    );
    collect_module_exports("models", &models, &mut external_defs);

    let consumer = compile(
        "consumer",
        r#"
from models import Wrapper

class Box:
    consumer_only: str = "wrong"

    @metadata("fixture.callback", "consumer")
    def consumer_method(self) -> str:
        return self.consumer_only

class Use:
    value: Wrapper
"#,
        &external_defs,
    );
    let imported = describe_type_with_externals(
        "consumer",
        field_type(&consumer, "Use", "value"),
        &consumer,
        &external_defs,
    );

    assert_eq!(local.canonical_identity, imported.canonical_identity);
    assert!(imported.canonical_identity.contains("models.Box"));
    assert!(!imported.canonical_identity.contains("consumer_only"));
    assert!(!imported.canonical_identity.contains("consumer_method"));
}

#[test]
fn public_import_preserves_private_generic_nested_shape() {
    let mut external_defs = ExternalDefs::default();
    let models = compile(
        "models",
        r#"
class _Hidden[T]:
    value: T = 0

    @metadata("fixture.callback", "read")
    def read(self) -> T:
        return self.value

class Box:
    hidden: _Hidden[int]

class LocalUse:
    item: Box
"#,
        &external_defs,
    );
    let local_shape = describe_type_with_externals(
        "models",
        field_type(&models, "LocalUse", "item"),
        &models,
        &external_defs,
    );
    collect_module_exports("models", &models, &mut external_defs);
    assert!(!external_defs.classes["models"].contains_key("_Hidden"));
    assert!(external_defs
        .structural_methods_for("models")
        .is_some_and(|classes| classes.contains_key("_Hidden")));

    let consumer = compile(
        "consumer",
        "from models import Box\n\nclass Container:\n    item: Box\n",
        &external_defs,
    );
    let shape = describe_type_with_externals(
        "consumer",
        field_type(&consumer, "Container", "item"),
        &consumer,
        &external_defs,
    );
    assert_eq!(local_shape.canonical_identity, shape.canonical_identity);
    assert!(shape.canonical_identity.contains("models._Hidden"));
    assert!(shape.canonical_identity.contains("value:int:default=int:0"));
    assert!(shape.canonical_identity.contains("read:regular"));
    assert!(shape.canonical_identity.contains("result[int]"));
    assert!(shape.canonical_identity.contains("fixture.callback"));
}
