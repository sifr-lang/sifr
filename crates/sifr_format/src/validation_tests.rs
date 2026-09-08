use super::*;

#[test]
fn validation_only_formatting_matches_full_syntax_projection_oracle() {
    let file = Path::new("nested/é.sifr");
    for source in [
        "def main( ):\n    pass\n",
        "def take(mut own values:list[int])->list[int]:\n    return values\n",
        "def query(x:int)->Template:\n    return t\"value = { x }\"\n",
        "value = \"é🦀\\n\"\n",
        "# fmt: off\nvalue=1\n# fmt: on\nother=2\n",
        "lazy import value\n",
        "def broken(:\n",
        "value = \"unterminated\n",
        "def f():\n  x = 1\n y = 2\n",
    ] {
        let options = FormatOptions::default();
        let expected =
            format_sifr_module_source(source, ruff_options(Some(file), options).unwrap())
                .map_err(|error| vec![format_module_error_diagnostic(source, Some(file), &error)])
                .and_then(|printed| {
                    let formatted = printed.into_code();
                    // Deliberately retain the original public full-syntax oracle.
                    sifr_syntax::parse_module(&formatted, Some("nested/é.sifr"))?;
                    Ok(FormatResult {
                        changed: formatted != source,
                        formatted,
                    })
                });
        assert_eq!(
            format_source(source, Some(file), options),
            expected,
            "{source:?}"
        );
    }
}

#[test]
fn range_validation_keeps_complete_roundtrip_and_outside_bytes() {
    let source = "# before\ndef main():\n    x=1\n    text = \"é🦀\"\n# after\n";
    let start = source.find("x=1").unwrap() as u32;
    let range = TextRange::new(TextSize::new(start), TextSize::new(start + 3));
    let edits = format_range(
        source,
        range,
        Some(Path::new("range.sifr")),
        FormatOptions::default(),
    )
    .expect("valid range");
    assert_eq!(edits.len(), 1);
    let edit = &edits[0];
    let output = source_with_edit(source, edit.range, &edit.replacement).expect("apply range");
    sifr_syntax::parse_module(&output, Some("range.sifr")).expect("full parser oracle");
    assert_eq!(output, source.replace("x=1", "x = 1"));
}

#[test]
fn invalid_ranges_keep_diagnostics_without_mutating_source() {
    let source = "value = \"é\"\n";
    let inside = source.find('é').unwrap() as u32 + 1;
    for range in [
        TextRange::new(TextSize::new(inside), TextSize::new(inside + 1)),
        TextRange::new(TextSize::new(0), TextSize::new(source.len() as u32 + 1)),
    ] {
        let errors = format_range(
            source,
            range,
            Some(Path::new("invalid.sifr")),
            FormatOptions::default(),
        )
        .expect_err("invalid byte boundary");
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "SIFR-FMT-0001");
    }
}
