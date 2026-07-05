const COMPLETED_MIGRATED_PRIVATE_DECLARATIONS: &[(&str, &str)] = &[
    (
        "_sifr.platform",
        include_str!("../../../../stdlib/_sifr/platform.sifr"),
    ),
    (
        "_sifr.html",
        include_str!("../../../../stdlib/_sifr/html.sifr"),
    ),
    (
        "_sifr.calendar",
        include_str!("../../../../stdlib/_sifr/calendar.sifr"),
    ),
    (
        "_sifr.uuid",
        include_str!("../../../../stdlib/_sifr/uuid.sifr"),
    ),
    (
        "_sifr.math",
        include_str!("../../../../stdlib/_sifr/math.sifr"),
    ),
    (
        "_sifr.crypto",
        include_str!("../../../../stdlib/_sifr/crypto.sifr"),
    ),
    (
        "_sifr.compress",
        include_str!("../../../../stdlib/_sifr/compress.sifr"),
    ),
    (
        "_sifr.datetime",
        include_str!("../../../../stdlib/_sifr/datetime.sifr"),
    ),
    (
        "_sifr.bytes",
        include_str!("../../../../stdlib/_sifr/bytes.sifr"),
    ),
    (
        "_sifr.collections",
        include_str!("../../../../stdlib/_sifr/collections.sifr"),
    ),
    (
        "_sifr.regex",
        include_str!("../../../../stdlib/_sifr/regex.sifr"),
    ),
    (
        "_sifr.url",
        include_str!("../../../../stdlib/_sifr/url.sifr"),
    ),
    (
        "_sifr.toml",
        include_str!("../../../../stdlib/_sifr/toml.sifr"),
    ),
    (
        "_sifr.json",
        include_str!("../../../../stdlib/_sifr/json.sifr"),
    ),
    (
        "_sifr.encoding",
        include_str!("../../../../stdlib/_sifr/encoding.sifr"),
    ),
    (
        "_sifr.unicode",
        include_str!("../../../../stdlib/_sifr/unicode.sifr"),
    ),
    (
        "_sifr.i18n",
        include_str!("../../../../stdlib/_sifr/i18n.sifr"),
    ),
];

#[test]
fn completed_private_declarations_follow_adapter_policy_syntax() {
    for (module, source) in COMPLETED_MIGRATED_PRIVATE_DECLARATIONS {
        assert!(
            !source.contains("@rust.via"),
            "{module} must not use callee-injection syntax"
        );
        assert!(
            !source.contains("bridge."),
            "{module} must not route through bridge.* sysroot adapters"
        );
        assert!(
            !source.contains("converter") && !source.contains("pipeline"),
            "{module} must not declare converter-pipeline metadata"
        );
        for line in source
            .lines()
            .filter(|line| line.trim_start().starts_with("@rust("))
        {
            assert!(
                line.contains("@rust(sifr_stdlib.") && line.contains("panic=trusted_no_panic"),
                "{module} declaration must bind directly to sifr_stdlib with sysroot trust: {line}"
            );
        }
    }
}
