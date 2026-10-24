"""Exercise the production Cargo input watcher without requiring the WASI SDK."""
from __future__ import annotations

import json
import os
import subprocess
import tempfile
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]


class PostgreSqlBuildInputsTests(unittest.TestCase):
    def test_cargo_tracks_headers_additions_removals_and_sdk_but_not_git(self) -> None:
        helper = ROOT / "crates/sifr_sql_postgresql/src/build_inputs.rs"
        toolchain = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
        with tempfile.TemporaryDirectory(prefix="postgresql-input-tracking-") as directory:
            root = Path(directory)
            native, compatibility = root / "native", root / "compat"
            headers = [
                native / "pg_query.h",
                native / "postgres_deparse.h",
                native / "src/include/native.h",
                native / "vendor/vendor.h",
                native / "protobuf/query.pb-c.h",
                compatibility / "errno.h",
            ]
            for index, header in enumerate(headers):
                header.parent.mkdir(parents=True, exist_ok=True)
                header.write_text(f"#define VALUE {index}\n")
            (root / "src").mkdir()
            (root / "Cargo.toml").write_text(
                '[package]\nname = "postgresql-input-tracking-probe"\n'
                'version = "0.0.0"\nedition = "2024"\n[workspace]\n'
            )
            (root / "src/main.rs").write_text(
                'include!(concat!(env!("OUT_DIR"), "/inputs.rs"));\n'
                'fn main() { println!("{INPUTS}"); }\n'
            )
            (root / "build.rs").write_text(
                f"#[path = {json.dumps(str(helper), ensure_ascii=False)}]\nmod inputs;\n"
                + r'''
use std::{fs, io::{self, Write}, path::Path};
fn collect(path: &Path, rows: &mut Vec<String>) -> io::Result<()> {
    if path.file_name().is_some_and(|name| name == ".git") { return Ok(()); }
    if path.is_dir() {
        let mut entries = fs::read_dir(path)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<io::Result<Vec<_>>>()?;
        entries.sort();
        for entry in entries { collect(&entry, rows)?; }
    } else {
        rows.push(format!("{}={}", path.to_string_lossy().replace('\\', "/"),
                          fs::read_to_string(path)?.trim()));
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    inputs::track_source(Path::new("native"))?;
    inputs::track_wasi(Path::new("compat"));
    let mut rows = Vec::new();
    collect(Path::new("native"), &mut rows)?;
    collect(Path::new("compat"), &mut rows)?;
    rows.sort();
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").ok_or("missing OUT_DIR")?);
    fs::write(output.join("inputs.rs"), format!("const INPUTS: &str = {:?};", rows.join("\n")))?;
    let mut count = fs::OpenOptions::new().create(true).append(true)
        .open(output.join("invocations"))?;
    writeln!(count, "run")?;
    Ok(())
}
'''
            )
            environment = {**os.environ, "CARGO_TARGET_DIR": str(root / "target"),
                           "RUSTUP_TOOLCHAIN": toolchain}
            result = subprocess.run(
                ["cargo", "generate-lockfile", "--offline"], cwd=root, env=environment,
                capture_output=True, text=True, timeout=60,
            )
            self.assertEqual(result.returncode, 0, result.stderr)

            def run(expected_invocations: int) -> None:
                result = subprocess.run(
                    ["cargo", "run", "--quiet", "--locked", "--offline"],
                    cwd=root, env=environment, capture_output=True, text=True, timeout=60,
                )
                self.assertEqual(result.returncode, 0, result.stderr)
                expected = sorted(
                    f"{path.relative_to(root).as_posix()}={path.read_text().strip()}"
                    for directory in (native, compatibility)
                    for path in directory.rglob("*")
                    if path.is_file() and ".git" not in path.relative_to(root).parts
                )
                self.assertEqual(result.stdout, "\n".join(expected) + "\n")
                counters = list((root / "target").rglob("invocations"))
                self.assertEqual(len(counters), 1)
                self.assertEqual(len(counters[0].read_text().splitlines()), expected_invocations)

            run(1)
            run(1)
            (native / ".git").mkdir()
            (native / ".git/index").write_text("irrelevant Git bookkeeping")
            run(1)
            count = 1
            for index, header in enumerate(headers):
                header.write_text(f"#define VALUE {index + 20}\n")
                count += 1
                run(count)
            added = native / "src/include/new.h"
            added.write_text("#define ADDED 1\n")
            count += 1
            run(count)
            added.unlink()
            count += 1
            run(count)
            environment["WASI_SDK_PATH"] = "different-sdk-selection"
            run(count + 1)


if __name__ == "__main__":
    unittest.main()
