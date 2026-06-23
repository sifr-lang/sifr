use sifr_driver::BuildReport;
use std::path::Path;
use std::time::Duration;

pub(super) struct BuildOutputOptions<'a> {
    pub(super) version: &'a str,
    pub(super) quiet: bool,
    pub(super) include_binary: bool,
}

pub(super) fn render_build_success(
    report: &BuildReport,
    options: &BuildOutputOptions<'_>,
) -> String {
    if options.quiet {
        return render_quiet_success(report, options.include_binary);
    }

    let mut output = String::new();
    output.push_str("sifr ");
    output.push_str(options.version);
    output.push('\n');
    push_key_value(&mut output, "input:", &quote_path(report.entrypoint_path()));
    push_key_value(&mut output, "mode:", report.mode().as_str());
    push_key_value(&mut output, "target:", report.target());
    push_key_value(
        &mut output,
        "sysroot:",
        &quote_path(report.sysroot().root()),
    );
    push_key_value(&mut output, "toolchain:", report.sysroot().toolchain_id());
    push_key_value(&mut output, "digest:", report.sysroot().content_sha256());
    output.push('\n');

    let label_width = report
        .stages()
        .iter()
        .map(|stage| stage.label().len())
        .max()
        .unwrap_or(0);
    let duration_width = report
        .stages()
        .iter()
        .map(|stage| format_duration(stage.elapsed()).len())
        .max()
        .unwrap_or(0);
    for stage in report.stages() {
        let duration = format_duration(stage.elapsed());
        output.push_str("   ");
        output.push_str(stage.label());
        output.push_str(&" ".repeat(label_width.saturating_sub(stage.label().len()) + 2));
        output.push_str(&" ".repeat(duration_width.saturating_sub(duration.len())));
        output.push_str(&duration);
        output.push('\n');
    }

    output.push('\n');
    output.push_str(&finished_line(report));
    output.push('\n');
    if options.include_binary {
        output.push_str("Binary: ");
        output.push_str(&quote_path(report.binary_path()));
        output.push('\n');
    }
    if let Some(size) = report.binary_size_bytes() {
        output.push_str("Size:   ");
        output.push_str(&format_binary_size(size));
        output.push('\n');
    }
    output
}

fn render_quiet_success(report: &BuildReport, include_binary: bool) -> String {
    let mut output = String::new();
    output.push_str(&finished_line(report));
    output.push('\n');
    if include_binary {
        output.push_str("Binary: ");
        output.push_str(&quote_path(report.binary_path()));
        output.push('\n');
    }
    output
}

fn push_key_value(output: &mut String, key: &str, value: &str) {
    output.push_str(key);
    output.push_str(&" ".repeat(8usize.saturating_sub(key.len())));
    output.push_str(value);
    output.push('\n');
}

fn finished_line(report: &BuildReport) -> String {
    let cached = if report.cache_hit() { " (cached)" } else { "" };
    format!(
        "Finished release build in {}{cached}",
        format_duration(report.total_elapsed())
    )
}

fn format_duration(duration: Duration) -> String {
    let millis = duration.as_millis();
    if millis < 1_000 {
        format!("{millis} ms")
    } else {
        let seconds = duration.as_secs_f64();
        format!("{seconds:.1} s")
    }
}

fn format_binary_size(size: u64) -> String {
    if size >= 1_000_000 {
        format_decimal_unit(size, 1_000_000, "MB")
    } else if size >= 1_000 {
        format_decimal_unit(size, 1_000, "KB")
    } else {
        format!("{size} B")
    }
}

fn format_decimal_unit(size: u64, unit: u64, suffix: &str) -> String {
    let tenths = (u128::from(size) * 10 + u128::from(unit / 2)) / u128::from(unit);
    format!("{}.{:01} {suffix}", tenths / 10, tenths % 10)
}

fn quote_path(path: &Path) -> String {
    let raw = path.display().to_string();
    if raw.chars().any(char::is_whitespace) || raw.contains('\'') {
        let escaped = raw.replace('\'', "'\\''");
        format!("'{escaped}'")
    } else {
        raw
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_driver::{BuildCompilationMode, BuildStageReport, BuildSysrootReport};

    fn report(cache_hit: bool) -> BuildReport {
        BuildReport::new(
            Path::new("demo main.sifr").to_path_buf(),
            BuildCompilationMode::Project,
            BuildSysrootReport::new(
                Path::new("/opt/sifr").to_path_buf(),
                "0.1.0-test-x86_64-test",
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            ),
            Path::new("./sifr_output/target/release/sifr_output").to_path_buf(),
            Duration::from_millis(54),
            vec![
                BuildStageReport::new("Loading Sifr standard library", Duration::from_millis(8)),
                BuildStageReport::new(
                    "Parsing import closure (4 modules)",
                    Duration::from_millis(3),
                ),
                BuildStageReport::new("Analyzing 4 modules", Duration::from_millis(12)),
                BuildStageReport::new("Generating Rust project", Duration::from_millis(4)),
                BuildStageReport::new("Materializing Cargo project", Duration::from_millis(1)),
                BuildStageReport::new("Building release binary", Duration::from_millis(26)),
            ],
            Vec::new(),
            cache_hit,
        )
    }

    #[test]
    fn build_output_default_includes_phase_summary() {
        let rendered = render_build_success(
            &report(false),
            &BuildOutputOptions {
                version: "0.1.0",
                quiet: false,
                include_binary: true,
            },
        );

        assert!(rendered.contains("sifr 0.1.0\n"));
        assert!(rendered.contains("input:  'demo main.sifr'\n"));
        assert!(rendered.contains("mode:   project\n"));
        assert!(rendered.contains("target: release native\n"));
        assert!(rendered.contains("sysroot:/opt/sifr\n"));
        assert!(rendered.contains("toolchain:0.1.0-test-x86_64-test\n"));
        assert!(rendered.contains(
            "digest: 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\n"
        ));
        assert!(rendered.contains("Loading Sifr standard library"));
        assert!(rendered.contains("Parsing import closure (4 modules)"));
        assert!(rendered.contains("Finished release build in 54 ms\n"));
        assert!(rendered.contains("Binary: ./sifr_output/target/release/sifr_output\n"));
    }

    #[test]
    fn build_output_quiet_is_two_lines() {
        let rendered = render_build_success(
            &report(false),
            &BuildOutputOptions {
                version: "0.1.0",
                quiet: true,
                include_binary: true,
            },
        );

        assert_eq!(
            rendered,
            "Finished release build in 54 ms\nBinary: ./sifr_output/target/release/sifr_output\n"
        );
    }

    #[test]
    fn build_output_cached_marks_finished_line() {
        let rendered = render_build_success(
            &report(true),
            &BuildOutputOptions {
                version: "0.1.0",
                quiet: true,
                include_binary: true,
            },
        );

        assert!(rendered.starts_with("Finished release build in 54 ms (cached)\n"));
    }
}
