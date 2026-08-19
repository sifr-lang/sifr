use sifr_frontend::SourceProvider;

pub(super) fn package_session_for_cwd(
    lock_mode: sifr_package::CargoLockMode,
    provider: &mut impl SourceProvider,
) -> Result<sifr_package::PackageSession, sifr_package::PackageDiagnostic> {
    let current_dir = std::env::current_dir().map_err(|error| {
        sifr_package::PackageDiagnostic::cargo_command_failed(
            sifr_package::CargoAction::Metadata,
            format!("could not read current directory: {error}"),
        )
    })?;
    sifr_package::PackageSession::discover(
        sifr_package::PackageSessionOptions {
            current_dir,
            lock_mode,
        },
        provider,
    )
}
