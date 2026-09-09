use super::*;
use sifr_python_parser::parse_module;

fn external_input() -> ExternalDefs {
    let mut defs = ExternalDefs::default();
    defs.constants
        .entry("values".into())
        .or_default()
        .insert("ANSWER".into(), Type::Int);
    defs.constant_integer_values
        .entry("values".into())
        .or_default()
        .insert("ANSWER".into(), 42.into());
    defs.attached_api_sets
        .entry("unrelated".into())
        .or_default()
        .insert(
            "Api".into(),
            sifr_ir::AttachedApiSetDeclaration {
                identity: sifr_ir::AttachedApiSetIdentity {
                    module: "unrelated".into(),
                    symbol: "Api".into(),
                },
                range: TextRange::default(),
            },
        );
    defs
}

#[test]
fn lowering_external_defs_context_borrows_complete_input() {
    let defs = external_input();
    let ctx = LowerCtx::new().with_external_defs(&defs);
    assert!(matches!(ctx.externals, Cow::Borrowed(_)));
    assert!(std::ptr::eq(ctx.externals.as_ref(), &defs));
    assert!(std::ptr::eq(&ctx.externals.constants, &defs.constants));
    assert!(std::ptr::eq(
        &ctx.externals.attached_api_sets,
        &defs.attached_api_sets
    ));
    assert!(
        ctx.externals
            .contains_attached_api_set(&sifr_ir::AttachedApiSetIdentity {
                module: "unrelated".into(),
                symbol: "Api".into(),
            })
    );
    let empty = LowerCtx::new();
    assert!(matches!(empty.externals, Cow::Owned(_)));
    assert_eq!(
        format!("{:?}", empty.externals),
        format!("{:?}", ExternalDefs::default())
    );
}

#[test]
fn lowering_external_defs_preserves_success_and_failure_inputs() {
    let defs = external_input();
    let before = format!("{defs:?}");
    let valid =
        parse_module("from values import ANSWER\ndef main() -> int:\n    return ANSWER\n").unwrap();
    let invalid =
        parse_module("from values import MISSING\ndef main() -> int:\n    return MISSING\n")
            .unwrap();
    for _ in 0..2 {
        for (suite, succeeds) in [(valid.suite(), true), (invalid.suite(), false)] {
            let results = [
                lower_module_with_externals(suite, &defs),
                lower_module_with_externals_and_name("consumer", suite, &defs),
                lower_module_with_externals_name_and_options(
                    "consumer",
                    suite,
                    &defs,
                    LoweringOptions::default(),
                ),
                lower_module_sysroot_public_stdlib_with_externals(suite, &defs),
                lower_module_sysroot_private_declaration_with_externals(suite, &defs),
            ];
            for result in results {
                assert_eq!(result.is_ok(), succeeds, "{:?}", result.err());
            }
            assert_eq!(format!("{defs:?}"), before);
        }
    }
    let declaration = parse_module(
        "@compiler_intrinsic(test_assert_true)\ndef assert_true(value: bool) -> None:\n    ...\n",
    )
    .unwrap();
    assert!(lower_module_sysroot_public_stdlib_with_externals(declaration.suite(), &defs).is_ok());
    assert!(
        lower_module_sysroot_private_declaration_with_externals(declaration.suite(), &defs)
            .is_err()
    );
    assert!(lower_module_with_externals(declaration.suite(), &defs).is_err());
    assert_eq!(format!("{defs:?}"), before);
}
