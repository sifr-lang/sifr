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
    /// Traverse every canonical metadata section and project all semantic/codegen payloads
    ValidateMetadata {
        #[arg(long, requires = "metadata")]
        source_root: Option<PathBuf>,
        #[arg(long, requires = "source_root")]
        metadata: Option<PathBuf>,
        #[arg(long, requires = "source_root")]
        target: Option<String>,
    },
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
        SysrootCommand::ValidateMetadata {
            source_root,
            metadata,
            target,
        } => {
            let target = target.unwrap_or_else(|| env!("SIFR_BUILD_TARGET").into());
            let result = match (source_root, metadata) {
                (Some(root), Some(metadata)) => sifr_driver::qualify_development_metadata(
                    &crate::compiler_identity(),
                    &root,
                    &metadata,
                    &target,
                ),
                (None, None) => {
                    sifr_driver::CompilerContext::new(crate::compiler_identity()).qualify_metadata()
                }
                _ => Err("source-root and metadata must be provided together".into()),
            };
            match result {
                Ok(report) => {
                    let _ = writeln!(io::stdout(), "{report}");
                    0
                }
                Err(error) => {
                    let _ = writeln!(io::stderr(), "{error}");
                    2
                }
            }
        }
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
