use super::*;
use sifr_frontend::DiskSourceProvider;

#[test]
fn formatter_discovery_infallible_alphabet_agrees_with_engine_validation() {
    // Exhaust the interactions between every admitted syntax operator and a
    // literal through length four, rather than just the repository's rules.
    let alphabet = [b'a', b'/', b'*', b'?', b'!', b'.', b'-'];
    let mut patterns = vec![String::new()];
    for _ in 0..4 {
        let mut next = Vec::new();
        for prefix in patterns {
            for byte in alphabet {
                let mut pattern = prefix.clone();
                pattern.push(char::from(byte));
                assert!(infallible_rule_syntax(&pattern));
                assert!(
                    GitignoreBuilder::new(".").add_line(None, &pattern).is_ok(),
                    "{pattern:?}"
                );
                next.push(pattern);
            }
        }
        patterns = next;
    }
    for source in [
        "", "[a-z]", "{a,b}", "\\!name", "name ", "é.sifr", "#comment",
    ] {
        assert!(!infallible_rule_syntax(source), "{source:?}");
    }
}

#[test]
fn formatter_discovery_validity_proof_keeps_matching_in_the_original_engine() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().canonicalize().unwrap();
    for source in [
        "**/target/\n!target/keep.sifr\n",
        "***a**?*.sifr\n!a.sifr\n",
        "!/root/**\n/root/*.sifr\n",
        "!\n/\n!!foo\n*\n!keep.sifr\n",
        "./a//b/\nfoo/**/**\nfoo/***\n",
        "[ab].sifr\n!b.sifr\n{foo,bar}/\n",
        include_str!("../../../.gitignore"),
    ] {
        std::fs::write(root.join(".gitignore"), source).unwrap();
        let matcher =
            FormatterGitignore::load(&root, true, &mut DiskSourceProvider::new()).unwrap();
        let mut oracle = GitignoreBuilder::new(&root);
        for line in source.lines() {
            oracle
                .add_line(Some(root.join(".gitignore")), line)
                .unwrap();
        }
        let oracle = oracle.build().unwrap();
        for path in [
            "main.sifr",
            "a.sifr",
            "b.sifr",
            "keep.sifr",
            "!foo",
            "root/main.sifr",
            "target/main.sifr",
            "target/keep.sifr",
            "nested/target/a.sifr",
            "foo/main.sifr",
            "bar/main.sifr",
            "foo/a/b/c",
            "a/b/main.sifr",
            "crates/sifr_driver/src/build/main.sifr",
            "demos/one/example.sifr",
            "demos/one/example.rs",
            "demos/one/example.o",
            "vendor/a/Cargo.lock",
        ] {
            let path = root.join(path);
            assert_eq!(
                matcher.matches_resolved_path(&path).unwrap(),
                oracle.matched_path_or_any_parents(&path, false).is_ignore(),
                "source {source:?}, path {path:?}"
            );
        }
    }
}

#[test]
fn formatter_discovery_still_validates_impossible_complex_rules_eagerly() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    for bad in ["unrelated/{bad", "unrelated/[z-a]", "unrelated/escape\\"] {
        std::fs::write(
            root.join(".gitignore"),
            format!("# comment\nvalid/**\n{bad}\n"),
        )
        .unwrap();
        let diagnostic = FormatterGitignore::load(root, true, &mut DiskSourceProvider::new())
            .err()
            .expect("malformed rule fails before path selection");
        assert_eq!(diagnostic.len(), 1);
        let engine_error = GitignoreBuilder::new(root)
            .add_line(Some(root.join(".gitignore")), bad)
            .err()
            .expect("independent original engine rejects the rule");
        assert_eq!(
            diagnostic[0].message,
            format!(
                "invalid formatter gitignore {}:3: {engine_error}",
                root.join(".gitignore").display()
            )
        );
    }
}
