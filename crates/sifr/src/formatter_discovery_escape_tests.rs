use super::*;
use sifr_frontend::DiskSourceProvider;

fn load(root: &Path, source: &str) -> FormatterGitignore {
    std::fs::write(root.join(".gitignore"), source).unwrap();
    FormatterGitignore::load(root, true, &mut DiskSourceProvider::new()).unwrap()
}

#[test]
fn formatter_discovery_escaped_literals_remain_uncompiled_when_impossible() {
    let dir = tempfile::tempdir().unwrap();
    let rules = [
        (r"\!notice.sifr", "!notice.sifr"),
        (r"\#notice.sifr", "#notice.sifr"),
        (r"literal\*.sifr", "literal*.sifr"),
        (r"literal\?.sifr", "literal?.sifr"),
        (r"literal\\name.sifr", r"literal\name.sifr"),
        ("literal\\ ", "literal "),
        (r"\épreuve.sifr", "épreuve.sifr"),
        (r"\{literal\}.sifr", "{literal}.sifr"),
    ];
    for (rule, literal) in rules {
        assert_eq!(mandatory_literal(rule).as_deref(), Some(literal));
        let matcher = load(dir.path(), rule);
        for path in ["main.sifr", "src/second.sifr"] {
            assert!(!matcher.matches_resolved_path(Path::new(path)).unwrap());
        }
        assert!(matcher.rules[0].compiled.get().is_none(), "{rule}");
        assert!(matcher.matches_resolved_path(Path::new(literal)).unwrap());
        assert!(matcher.rules[0].compiled.get().is_some(), "{rule}");
    }
}

#[test]
fn formatter_discovery_escape_oracle_matches_engine() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let rules = [
        r"\!notice.sifr",
        r"\#notice.sifr",
        r"literal\*.sifr",
        r"literal\?.sifr",
        r"literal\\name.sifr",
        "literal\\ ",
        "literal   ",
        r"\épreuve.sifr",
        r"\{literal\}.sifr",
        r"\[literal\].sifr",
        r"literal\/",
        r"/literal\/",
        r"literal\/**",
        r"literal\ space/*.sifr",
        r"*/\!notice.sifr",
        r"literal/**/\?.sifr",
        "*.sifr\n!\\!notice.sifr",
        "!\\!notice.sifr\nliteral/",
        "literal/\n!\\!notice.sifr",
        "[lm]*.sifr",
        "{literal,main}.sifr",
        "[unclosed",
        "\\!notice.sifr\n!*.sifr\n\\!notice.sifr",
        "\u{feff}\\!notice.sifr",
    ];
    let paths = [
        "main.sifr",
        "src/main.sifr",
        "!notice.sifr",
        "#notice.sifr",
        "literal*.sifr",
        "literal?.sifr",
        r"literal\name.sifr",
        "literal ",
        "literal",
        "literal/child.sifr",
        "literal/!notice.sifr",
        "src/!notice.sifr",
        "literal/?.sifr",
        "literal/sub/?.sifr",
        "literal space/main.sifr",
        "épreuve.sifr",
        "{literal}.sifr",
        "[literal].sifr",
        "[unclosed",
        "sub/[unclosed",
        "literal.sifr",
    ];
    for source in rules {
        let matcher = load(&root, source);
        let mut builder = GitignoreBuilder::new(&root);
        for (index, line) in source.lines().enumerate() {
            let line = if index == 0 {
                line.trim_start_matches('\u{feff}')
            } else {
                line
            };
            builder.add_line(None, line).unwrap();
        }
        let oracle = builder.build().unwrap();
        for path in paths {
            let absolute = root.join(path);
            assert_eq!(
                matcher.matches_resolved_path(&absolute).unwrap(),
                oracle
                    .matched_path_or_any_parents(&absolute, false)
                    .is_ignore(),
                "rule {source:?}, path {path:?}"
            );
        }
    }
}

#[test]
fn formatter_discovery_invalid_escaped_rule_is_not_skipped() {
    let dir = tempfile::tempdir().unwrap();
    for rule in ["unrelated\\", r"unrelated\*}"] {
        std::fs::write(dir.path().join(".gitignore"), rule).unwrap();
        let result = FormatterGitignore::load(dir.path(), true, &mut DiskSourceProvider::new());
        assert!(
            result.is_err(),
            "original rule validation must reject {rule:?}"
        );
    }
}
