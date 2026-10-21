use super::{CheckedModuleObserver, check_stdlib_sources};
use crate::diagnostics::{RenderedDiagnostic, diagnostic_with_code};
use sifr_diagnostics::DiagnosticCode;
use sifr_ir::HirModule;
use sifr_stdlib_manifest::{LoadedStdlibSource, LoadedStdlibSourceKind};
use std::collections::HashSet;
use std::sync::Arc;

#[derive(Default)]
struct RecordingObserver {
    events: Vec<String>,
    retained: Vec<Arc<HirModule>>,
    fail_on: Option<&'static str>,
}

impl<'a> CheckedModuleObserver<'a> for RecordingObserver {
    fn retain_declaration(&mut self, source: &'a LoadedStdlibSource, module: &Arc<HirModule>) {
        self.events.push(format!("retain:{}", source.module));
        self.retained.push(Arc::clone(module));
    }

    fn materialize(
        &mut self,
        source: &'a LoadedStdlibSource,
        _module: &HirModule,
        _transitive_deps: &HashSet<String>,
    ) -> Result<(), Vec<RenderedDiagnostic>> {
        self.events.push(format!("materialize:{}", source.module));
        if self.fail_on == Some(source.module.as_str()) {
            return Err(vec![diagnostic_with_code(
                "observer failure".to_owned(),
                DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE,
            )]);
        }
        Ok(())
    }
}

fn source(module: &str, source: &str) -> LoadedStdlibSource {
    LoadedStdlibSource {
        module: module.to_owned(),
        source: source.to_owned(),
        path: format!("{}.sifr", module.replace('.', "/")).into(),
        kind: if module.starts_with("_sifr.") {
            LoadedStdlibSourceKind::PrivateDeclaration
        } else {
            LoadedStdlibSourceKind::Public
        },
    }
}

#[test]
fn checked_observer_preserves_source_order_imports_and_canonical_hir() {
    let sources = [
        source("_sifr.zeta", "def value() -> int:\n    return 7\n"),
        source(
            "sifr.alpha",
            "from _sifr.zeta import value\ndef forwarded() -> int:\n    return value()\n",
        ),
        source(
            "sifr.beta",
            "from sifr.alpha import forwarded\ndef result() -> int:\n    return forwarded()\n",
        ),
    ];
    let mut observer = RecordingObserver::default();
    let checked = check_stdlib_sources(&sources, &mut observer).expect("checked source chain");
    assert_eq!(
        observer.events,
        [
            "retain:_sifr.zeta",
            "materialize:_sifr.zeta",
            "retain:sifr.alpha",
            "materialize:sifr.alpha",
            "retain:sifr.beta",
            "materialize:sifr.beta",
        ]
    );
    for (source, retained) in sources.iter().zip(&observer.retained) {
        assert!(Arc::ptr_eq(retained, &checked.hir_modules[&source.module]));
    }
    assert!(checked.defs.functions["sifr.beta"].contains_key("result"));
    assert_eq!(
        checked.transitive_deps["sifr.beta"],
        HashSet::from(["_sifr.zeta".to_owned(), "sifr.alpha".to_owned(),])
    );
    // Reversing the canonical source order must fail; no speculative exports.
    let mut reversed = sources.clone();
    reversed.reverse();
    let mut observer = RecordingObserver::default();
    assert!(check_stdlib_sources(&reversed, &mut observer).is_err());
    assert!(observer.events.is_empty());
}

#[test]
fn checked_observer_retains_empty_private_hir_and_interop_declarations() {
    let sources = [
        source("_sifr.empty", "# empty private declaration\n"),
        source(
            "_sifr.bridge",
            "@rust(sifr_stdlib.bridge)\ndef bridge() -> int: ...\n",
        ),
    ];
    let mut observer = RecordingObserver::default();
    let checked = check_stdlib_sources(&sources, &mut observer).expect("private declarations");
    assert_eq!(
        observer.events,
        [
            "retain:_sifr.empty",
            "retain:_sifr.bridge",
            "materialize:_sifr.bridge",
        ]
    );
    assert!(Arc::ptr_eq(
        &observer.retained[0],
        &checked.hir_modules["_sifr.empty"]
    ));
    assert!(checked.hir_modules["_sifr.empty"].functions.is_empty());
    assert!(
        !checked.hir_modules["_sifr.bridge"].functions[0]
            .rust_interop
            .is_empty()
    );
    assert!(!checked.defs.functions.contains_key("_sifr.empty"));
}

#[test]
fn checked_observer_failure_precedes_later_parse_error() {
    let sources = [
        source("sifr.first", "def first() -> int:\n    return 1\n"),
        source("sifr.broken", "def broken(:\n"),
    ];
    let mut observer = RecordingObserver {
        fail_on: Some("sifr.first"),
        ..Default::default()
    };
    let errors = match check_stdlib_sources(&sources, &mut observer) {
        Ok(_) => panic!("observer must fail"),
        Err(errors) => errors,
    };
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].message, "observer failure");
    assert_eq!(
        observer.events,
        ["retain:sifr.first", "materialize:sifr.first"]
    );
}

#[test]
fn checked_parse_and_lower_failures_never_reach_observer() {
    for text in ["def broken(:\n", "from _sifr.absent import missing\n"] {
        let sources = [source("sifr.broken", text)];
        let mut observer = RecordingObserver::default();
        let errors = match check_stdlib_sources(&sources, &mut observer) {
            Ok(_) => panic!("invalid source must fail"),
            Err(errors) => errors,
        };
        assert!(!errors.is_empty());
        assert!(
            errors
                .iter()
                .all(|error| error.code == DiagnosticCode::STDLIB_BOOTSTRAP_FAILURE.code())
        );
        assert!(observer.events.is_empty());
    }
}
