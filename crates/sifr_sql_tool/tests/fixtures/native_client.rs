//! Test-only native invocation: the operations are deliberately not SQL semantics.
use super::SOURCE;
use sifr_sql_tool::native_compiler::NativeCompiler;
use std::process::ExitCode;

pub(super) fn run(mode: &str) -> ExitCode {
    match execute(mode) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
fn execute(mode: &str) -> Result<(), String> {
    let root = std::env::current_dir().map_err(|error| error.to_string())?;
    let compiler = NativeCompiler::new(&root, SOURCE)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .map_err(|error| error.to_string())?;
    let result = runtime.block_on(compiler.run(async {
        if mode == "native-unused" {
            return Ok(());
        }
        let declarations = compiler.migration("fixture source")?;
        if declarations.len() != 1 || declarations[0].id.as_str() != "fixture" {
            return Err("unexpected fixture response".to_string());
        }
        if mode == "native-idle" {
            let ready = std::env::var_os("SIFR_FIXTURE_READY").ok_or("missing ready file")?;
            std::fs::write(ready, b"ready").map_err(|error| error.to_string())?;
            std::future::pending::<()>().await;
        }
        Ok(())
    }));
    match result {
        Ok(outcome) => outcome?,
        Err(signal) => std::process::exit(128 + signal),
    }
    compiler.finish()
}
