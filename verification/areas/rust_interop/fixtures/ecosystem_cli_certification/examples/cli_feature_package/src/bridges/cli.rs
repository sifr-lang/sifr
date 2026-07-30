use anyhow::Context as _;
use sifr_runtime::interop::RustPanicErrorBridge;
use std::fmt;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use tracing_subscriber::fmt::MakeWriter;

#[derive(Debug)]
pub struct CliErrorBridge {
    pub message: String,
}

impl fmt::Display for CliErrorBridge {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CliErrorBridge {}

pub fn parse_and_trace(args: &[String]) -> Result<String, CliErrorBridge> {
    execute_cli_probe(args).map_err(|error| CliErrorBridge {
        message: if error.to_string().contains("clap parse failed") {
            "clap parse failed through the anyhow adapter".to_owned()
        } else {
            "CLI tooling failed through the anyhow adapter".to_owned()
        },
    })
}

pub fn map_panic(error: RustPanicErrorBridge) -> CliErrorBridge {
    CliErrorBridge {
        message: error.to_string(),
    }
}

fn execute_cli_probe(args: &[String]) -> anyhow::Result<String> {
    let matches = clap::Command::new("sifr")
        .disable_help_flag(true)
        .arg(
            clap::Arg::new("mode")
                .long("mode")
                .required(true)
                .value_parser(["check", "build"]),
        )
        .try_get_matches_from(args)
        .context("clap parse failed")?;
    let mode = matches
        .get_one::<String>("mode")
        .context("clap omitted the required mode")?;

    let filter = tracing_subscriber::EnvFilter::try_new("sifr_cli_probe=trace")
        .context("tracing-subscriber env-filter rejected the directive")?;
    let capture = CaptureWriter::default();
    let captured_bytes = Arc::clone(&capture.bytes);
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(capture)
        .with_ansi(false)
        .without_time()
        .with_target(false)
        .finish();
    tracing::subscriber::with_default(subscriber, || {
        tracing::info!(
            target: "sifr_cli_probe",
            mode = mode.as_str(),
            "cli bridge event"
        );
        tracing::warn!(
            target: "sifr_cli_noise",
            mode = mode.as_str(),
            "excluded bridge event"
        );
    });
    let trace = {
        let bytes = captured_bytes
            .lock()
            .map_err(|_| anyhow::anyhow!("tracing capture lock was poisoned"))?;
        String::from_utf8(bytes.clone()).context("tracing output was not UTF-8")?
    };
    anyhow::ensure!(
        trace.contains("cli bridge event")
            && trace.contains(mode)
            && !trace.contains("excluded bridge event"),
        "filtered tracing event was not observed"
    );

    Ok(format!(
        "clap=4.6.1;mode={mode};tracing=0.1.44;subscriber=0.3.23;env-filter=enabled;event=observed;anyhow=1.0.102;adapter=CliError"
    ))
}

#[derive(Clone, Default)]
struct CaptureWriter {
    bytes: Arc<Mutex<Vec<u8>>>,
}

struct CaptureGuard {
    bytes: Arc<Mutex<Vec<u8>>>,
}

impl<'writer> MakeWriter<'writer> for CaptureWriter {
    type Writer = CaptureGuard;

    fn make_writer(&'writer self) -> Self::Writer {
        CaptureGuard {
            bytes: Arc::clone(&self.bytes),
        }
    }
}

impl Write for CaptureGuard {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let mut bytes = self
            .bytes
            .lock()
            .map_err(|_| io::Error::other("tracing capture lock was poisoned"))?;
        bytes.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
