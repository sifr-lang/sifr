use std::fs;
use std::path::{Path as StdPath, PathBuf};
use std::process::Command;

fn lt3(x: &i64) -> bool {
    *x < 3
}

fn accumulate(values: impl IntoIterator<Item = i64>) -> Vec<i64> {
    values
        .into_iter()
        .scan(0, |state, value| {
            *state += value;
            Some(*state)
        })
        .collect()
}

fn compress(
    values: impl IntoIterator<Item = i64>,
    selectors: impl IntoIterator<Item = bool>,
) -> Vec<i64> {
    values
        .into_iter()
        .zip(selectors)
        .filter_map(|(value, keep)| keep.then_some(value))
        .collect()
}

fn takewhile(predicate: fn(&i64) -> bool, values: impl IntoIterator<Item = i64>) -> Vec<i64> {
    values.into_iter().take_while(predicate).collect()
}

struct Path {
    inner: PathBuf,
}

impl Path {
    fn new(path: String) -> Self {
        Self { inner: path.into() }
    }

    fn iterdir(&self) -> Vec<String> {
        let mut entries = fs::read_dir(&self.inner)
            .into_iter()
            .flat_map(|items| items.flatten())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect::<Vec<_>>();
        entries.sort();
        entries
    }
}

fn write_text(path: &str, content: &str) -> std::io::Result<()> {
    fs::write(path, content)
}

fn run_command(command: &str) -> std::io::Result<String> {
    let output = Command::new("sh").arg("-c").arg(command).output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn getpid() -> u32 {
    std::process::id()
}

fn main() {
    let nums = [1, 2, 3, 4];
    println!(
        "{:?}",
        nums.iter()
            .copied()
            .skip(1)
            .take(3)
            .step_by(2)
            .collect::<Vec<_>>()
    );
    println!("{:?}", accumulate(nums.iter().copied()));
    println!(
        "{:?}",
        compress(
            nums.iter().copied(),
            [true, false, true, false].iter().copied(),
        )
    );
    println!("{:?}", takewhile(lt3, nums.iter().copied()));

    let base = format!("/tmp/sifr_itertools_iterables{}", getpid());
    let _ = run_command(&format!("mkdir -p {base}"));
    let result = (|| -> std::io::Result<usize> {
        write_text(&format!("{base}/a.txt"), "demo")?;
        write_text(&format!("{base}/b.txt"), "demo")?;
        let root = Path::new(base.clone());
        Ok(root.iterdir().into_iter().take(1).count())
    })();

    match result {
        Ok(length) => println!("{length}"),
        Err(err) => println!("ioerror: {err}"),
    }

    if StdPath::new(&base).exists() {
        let _ = run_command(&format!("rm -rf {base}"));
    }
}
