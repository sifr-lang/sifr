use std::{
    io::{self, Write},
    path::PathBuf,
    sync::atomic::AtomicBool,
};
#[derive(clap::Args)]
pub(crate) struct SysrootArgs {
    #[command(subcommand)]
    command: SysrootCommand,
}
#[derive(clap::Subcommand)]
enum SysrootCommand {
    /// Produce and validate indexed stdlib metadata from a complete source tree
    BuildMetadata {
        #[arg(long)]
        source_root: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long,default_value=env!("SIFR_BUILD_TARGET"))]
        target: String,
    },
}
pub(crate) fn run(args: SysrootArgs) -> i32 {
    match args.command {
        SysrootCommand::BuildMetadata {
            source_root,
            output,
            target,
        } => {
            let started = std::time::Instant::now();
            let result = sifr_driver::metadata_producer::ensure_development_metadata(
                &crate::compiler_identity(),
                &source_root,
                &target,
                &sifr_driver::cache_storage::root(),
                &AtomicBool::new(false),
            )
            .and_then(|metadata| metadata.publish_output(&output).map(|()| metadata));
            match result {
                Ok(metadata) => {
                    let _ = writeln!(
                        io::stdout(),
                        "{}",
                        serde_json::json!({"schema_version":1,"compiler_identity":crate::compiler_identity().as_str(),"semantic_target":target,"metadata_id":metadata.metadata_id,"production_seconds":metadata.production_seconds,"output":output,"cache_path":metadata.path,"elapsed_seconds":started.elapsed().as_secs_f64()})
                    );
                    0
                }
                Err(error) => {
                    let _ = writeln!(io::stderr(), "{error}");
                    2
                }
            }
        }
    }
}
