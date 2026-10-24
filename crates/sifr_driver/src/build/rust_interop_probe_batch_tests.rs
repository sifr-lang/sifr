use super::{protocol, workspace};
use serde_json::json;
use std::path::PathBuf;

fn artifact(path: &str) -> serde_json::Value {
    json!({"reason":"compiler-artifact","target":{"src_path":path}})
}
fn finished(success: bool) -> serde_json::Value {
    json!({"reason":"build-finished","success":success})
}
fn bytes(rows: &[serde_json::Value]) -> Vec<u8> {
    rows.iter()
        .map(|r| format!("{r}\n"))
        .collect::<String>()
        .into_bytes()
}

#[test]
fn every_member_requires_its_own_cargo_observation() {
    let members = vec![PathBuf::from("/a/lib.rs"), PathBuf::from("/b/lib.rs")];
    assert!(
        protocol::outcomes(
            &bytes(&[artifact("/a/lib.rs"), finished(true)]),
            b"",
            true,
            &members
        )
        .is_err()
    );
    let rows = bytes(&[artifact("/a/lib.rs"), artifact("/b/lib.rs"), finished(true)]);
    assert!(
        protocol::outcomes(&rows, b"", true, &members)
            .unwrap()
            .iter()
            .all(Result::is_ok)
    );
    let duplicate = vec![members[0].clone(), members[0].clone()];
    assert_eq!(
        protocol::outcomes(
            &bytes(&[artifact("/a/lib.rs"), finished(true)]),
            b"",
            true,
            &duplicate
        )
        .unwrap()
        .len(),
        2
    );
}

#[test]
fn protocol_damage_cannot_authorize_a_probe() {
    let members = vec![PathBuf::from("/a/lib.rs")];
    for rows in [
        b"{".to_vec(),
        Vec::new(),
        bytes(&[artifact("/a/lib.rs")]),
        bytes(&[artifact("/a/lib.rs"), finished(false)]),
        bytes(&[artifact("/a/lib.rs"), finished(true), finished(true)]),
        bytes(&[finished(true), artifact("/a/lib.rs")]),
        bytes(&[json!({"reason":"unknown"}), finished(true)]),
    ] {
        assert!(protocol::outcomes(&rows, b"", true, &members).is_err());
    }
    assert!(protocol::outcomes(b"", b"", false, &members).is_err());
    assert!(
        protocol::outcomes(b"", b"error: locked resolution failed", false, &members).unwrap()[0]
            .is_err()
    );
}

#[test]
fn failures_keep_their_owner_without_hiding_successful_members() {
    let members = vec![PathBuf::from("/a/lib.rs"), PathBuf::from("/b/lib.rs")];
    let error = json!({"reason":"compiler-message","target":{"src_path":"/b/lib.rs"},
        "message":{"level":"error","rendered":"error: failed b"}});
    let rows = bytes(&[artifact("/a/lib.rs"), error.clone(), finished(false)]);
    let result = protocol::outcomes(&rows, b"", false, &members).unwrap();
    assert!(result[0].is_ok());
    assert_eq!(result[1], Err("error: failed b".into()));
    assert!(
        protocol::outcomes(
            &bytes(&[artifact("/a/lib.rs"), error, finished(true)]),
            b"",
            true,
            &members
        )
        .is_err()
    );
}

const MANIFEST: &str = "[package]\nname = \"sifr-rust-probe\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[workspace]\n[dependencies]\n[profile.dev]\ndebug=0\n";

#[test]
fn workspace_preserves_original_crate_scope_and_independent_failures() {
    let temp = tempfile::tempdir().unwrap();
    let good = "pub struct Local; pub fn probe(_: crate::Local) {}";
    let bad_a = "pub struct Local; pub fn probe(_: crate::Local) { let _: bool = String::new(); }";
    let bad_b = "pub struct Local; pub fn probe(_: crate::Local) { let _: u32 = false; }";
    let mut previous = None;
    for (label, sources, expected) in [
        (
            "positive",
            vec![
                good,
                "pub struct Local; pub fn probe(_: crate::Local) { let _: u32 = 1; }",
            ],
            true,
        ),
        ("negative", vec![good, bad_a, bad_b], false),
        ("restored", vec![good, good], true),
    ] {
        let root = temp.path().join(label);
        let pairs = sources.iter().map(|s| (MANIFEST, *s)).collect::<Vec<_>>();
        let members = workspace::write(&root, &pairs).unwrap();
        for (member, source) in members.iter().zip(&sources) {
            assert_eq!(std::fs::read_to_string(member).unwrap(), *source);
        }
        let output = std::process::Command::new("cargo")
            .args([
                "check",
                "--workspace",
                "--keep-going",
                "--offline",
                "--quiet",
                "--message-format=json",
            ])
            .arg("--manifest-path")
            .arg(root.join("Cargo.toml"))
            .env("CARGO_TARGET_DIR", temp.path().join("target"))
            .output()
            .unwrap();
        assert_eq!(
            output.status.success(),
            expected,
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let outcomes =
            protocol::outcomes(&output.stdout, &output.stderr, expected, &members).unwrap();
        assert_eq!(outcomes.len(), sources.len());
        assert!(outcomes[0].is_ok());
        if !expected {
            assert!(outcomes[1..].iter().all(Result::is_err));
            assert_ne!(outcomes[1], outcomes[2]);
        } else {
            assert!(outcomes.iter().all(Result::is_ok));
        }
        if let Some(old) = previous {
            assert_ne!(members[0], old);
        }
        previous = Some(members[0].clone());
    }
}

#[test]
fn different_feature_manifests_cannot_share_a_workspace() {
    let temp = tempfile::tempdir().unwrap();
    let other = format!("{MANIFEST}\n[features]\nextra=[]\n");
    assert!(workspace::write(temp.path(), &[(MANIFEST, ""), (&other, "")]).is_err());
}

#[test]
fn macro_stdout_is_not_cargo_evidence() {
    let members = vec![PathBuf::from("/a/lib.rs")];
    let text = b"ordinary macro output\n{\n{\"log\":\"macro data\"}\n\xff\n";
    let mut complete = text.to_vec();
    complete.extend(bytes(&[artifact("/a/lib.rs"), finished(true)]));
    assert!(protocol::outcomes(&complete, b"", true, &members).unwrap()[0].is_ok());
    let mut missing_artifact = text.to_vec();
    missing_artifact.extend(bytes(&[finished(true)]));
    assert!(protocol::outcomes(&missing_artifact, b"", true, &members).is_err());
    let mut missing_completion = text.to_vec();
    missing_completion.extend(bytes(&[artifact("/a/lib.rs")]));
    assert!(protocol::outcomes(&missing_completion, b"", true, &members).is_err());
}
