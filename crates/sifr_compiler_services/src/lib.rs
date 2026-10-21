//! Lower compiler capabilities shared by the driver and editor.
mod diagnostics;
pub mod editor;
pub use sifr_semantic_services::export_policy;
pub mod metadata;
pub use sifr_semantic_services::private_re_exports;
pub mod python;
pub use sifr_semantic_services::sql_diagnostics;
pub use sifr_semantic_services::sql_editor;
pub mod stdlib;

mod application_profile;
pub use application_profile::ApplicationProfile;
mod compiler_context;
pub use compiler_context::{CompilerContext, compiled_input_tokens};
