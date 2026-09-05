// src/main.rs
mod sifr_generated_generated_support {
    use crate::{
        SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticError,
        SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticEvent,
        SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticLevel,
    };
    pub(crate) fn runtime_emit_diagnostic(
        level: &str,
        target: &str,
        name: &str,
        message: &str,
    ) -> Result<(), SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticError> {
        ::sifr_stdlib::runtime_observability::emit_diagnostic(level, target, name, message).map_err(
            |sifr_generated_bridge_error| SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticError {
                message: sifr_generated_bridge_error.to_string(),
            },
        )
    }
    pub(crate) fn sifr_generated_const_494e464f()
    -> SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticLevel {
        SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticLevel::new("info".to_string())
    }
    pub(crate) fn diagnostic_event(
        level: &SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticLevel,
        target: &str,
        name: &str,
        message: &str,
    ) -> SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticEvent {
        SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticEvent::new(
            level.clone(),
            target.to_owned(),
            name.to_owned(),
            message.to_owned(),
        )
    }
    pub(crate) fn emit_diagnostic(
        event: &SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticEvent,
    ) -> Result<(), SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticError> {
        runtime_emit_diagnostic(
            &event.level.clone().name.clone(),
            &event.target.clone(),
            &event.name.clone(),
            &event.message.clone(),
        )
    }
}
mod sifr_generated_project_nominals {
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticError {
        pub message: String,
    }
    impl ::std::fmt::Debug for SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            f.debug_struct("DiagnosticError")
                .field("message", &self.message)
                .finish()
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }
    impl ::std::error::Error for SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticLevel {
        pub name: String,
    }
    impl SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticLevel {
        #[must_use]
        pub const fn new(name: String) -> Self {
            let sifr_generated_field_value_c4bcadba8e631b86_6e616d65: String = name;
            Self {
                name: sifr_generated_field_value_c4bcadba8e631b86_6e616d65,
            }
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticLevel {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.name.clone())
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticEvent {
        pub level: SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticLevel,
        pub target: String,
        pub name: String,
        pub message: String,
    }
    impl SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticEvent {
        #[must_use]
        pub const fn new(
            level: SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticLevel,
            target: String,
            name: String,
            message: String,
        ) -> Self {
            let sifr_generated_field_value_e8ddc90a9d7c709d_6c6576656c: SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticLevel = level;
            let sifr_generated_field_value_16f3e46051eee3e8_746172676574: String = target;
            let sifr_generated_field_value_c4bcadba8e631b86_6e616d65: String = name;
            let sifr_generated_field_value_546401b5d2a8d2a4_6d657373616765: String = message;
            Self {
                level: sifr_generated_field_value_e8ddc90a9d7c709d_6c6576656c,
                target: sifr_generated_field_value_16f3e46051eee3e8_746172676574,
                name: sifr_generated_field_value_c4bcadba8e631b86_6e616d65,
                message: sifr_generated_field_value_546401b5d2a8d2a4_6d657373616765,
            }
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticEvent {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f,
                "DiagnosticEvent(level={}, target={}, name={}, message={})",
                self.level, self.target, self.name, self.message
            )
        }
    }
}
use crate::sifr_generated_generated_support::*;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticEvent;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticLevel;
#[expect(
    clippy::assertions_on_constants,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
fn main() {
    let event: SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticEvent = diagnostic_event(
        &sifr_generated_const_494e464f(),
        &"sifr.demo".to_string(),
        &"accepted".to_string(),
        &"stdlib boundary".to_string(),
    );
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticError> =
        (|| {
            emit_diagnostic(&event)?;
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        assert!(false);
    }
    let invalid: SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticEvent = diagnostic_event(
        &SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticLevel::new("verbose".to_string()),
        &"sifr.demo".to_string(),
        &"rejected".to_string(),
        &"stdlib boundary".to_string(),
    );
    let mut rejected: bool = false;
    let sifr_generated_try_res: Result<(), SifrGeneratedStdlibSifrX2eruntimeX2eDiagnosticError> =
        (|| {
            emit_diagnostic(&invalid)?;
            Ok(())
        })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let error = sifr_generated_try_err.clone();
        rejected = error.message.clone() == "unsupported diagnostic level: verbose";
    }
    assert!(rejected);
}
