//! Lower compiler capabilities shared by the driver and editor.
mod diagnostics;
pub mod export_policy;
pub mod metadata;
pub mod private_re_exports;
pub mod stdlib;

mod application_profile;
pub use application_profile::ApplicationProfile;
mod compiler_context;
pub use compiler_context::CompilerContext;
