use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

fn monotonic_clock() -> f64 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_secs_f64()
}

fn default_timer() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

fn timeit(mut work: impl FnMut(), iterations: usize) -> f64 {
    let start = Instant::now();
    for _ in 0..iterations {
        work();
    }
    start.elapsed().as_secs_f64()
}

fn repeat(mut work: impl FnMut(), repeats: usize, iterations: usize) -> Vec<f64> {
    (0..repeats)
        .map(|_| timeit(&mut work, iterations))
        .collect()
}

fn matches_glob(name: &str, pattern: &str) -> bool {
    fn inner(name: &[u8], pattern: &[u8]) -> bool {
        match pattern.split_first() {
            None => name.is_empty(),
            Some((&b'*', rest)) => (0..=name.len()).any(|index| inner(&name[index..], rest)),
            Some((&b'?', rest)) => !name.is_empty() && inner(&name[1..], rest),
            Some((&expected, rest)) => {
                !name.is_empty() && name[0] == expected && inner(&name[1..], rest)
            }
        }
    }

    inner(name.as_bytes(), pattern.as_bytes())
}

fn glob(directory: &str, pattern: &str) -> Vec<String> {
    let mut matches = fs::read_dir(directory)
        .into_iter()
        .flat_map(|entries| entries.flatten())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| matches_glob(name, pattern))
        .collect::<Vec<_>>();
    matches.sort();
    matches
}

fn write_text(path: &str, content: &str) -> std::io::Result<()> {
    fs::write(path, content)
}

fn exists(path: &str) -> bool {
    Path::new(path).exists()
}

fn copy(src: &str, dst: &str) -> std::io::Result<()> {
    fs::copy(src, dst).map(|_| ())
}

fn move_file(src: &str, dst: &str) -> std::io::Result<()> {
    fs::rename(src, dst)
}

fn rmtree(path: &str) -> std::io::Result<()> {
    fs::remove_dir_all(path)
}

fn loads(input: &str) -> Result<BTreeMap<String, String>, String> {
    let (key, value) = input
        .split_once('=')
        .ok_or_else(|| "missing key/value separator".to_string())?;
    let key = key.trim();
    let value = value.trim();
    if value.len() < 2 || !value.starts_with('"') || !value.ends_with('"') {
        return Err("expected quoted string".to_string());
    }

    let mut table = BTreeMap::new();
    table.insert(key.to_string(), value[1..value.len() - 1].to_string());
    Ok(table)
}

fn do_work() {
    let mut total = 0;
    let mut index = 0;
    while index < 1000 {
        total += index;
        index += 1;
    }
    let _ = total;
}

fn main() {
    let base = "/tmp/sifr_polish_demo";
    let _ = fs::remove_dir_all(base);

    println!("=== Monotonic Clocks ===");
    let t1 = monotonic_clock();
    let t2 = monotonic_clock();
    println!("{}", t2 >= t1);

    let monotonic_start = monotonic_clock();
    let monotonic_end = monotonic_clock();
    println!("{}", monotonic_end >= monotonic_start);

    println!("=== timeit (Callable API) ===");
    println!("{}", default_timer() >= 0.0);
    println!("{}", timeit(do_work, 100) >= 0.0);
    println!("{}", repeat(do_work, 3, 50).len());

    println!("=== glob ===");
    fs::create_dir_all(base).unwrap();
    write_text(&format!("{base}/a.txt"), "aaa").unwrap();
    write_text(&format!("{base}/b.txt"), "bbb").unwrap();
    write_text(&format!("{base}/c.csv"), "1,2").unwrap();
    println!("{}", glob(base, "*.txt").len());

    println!("=== shutil ===");
    copy(&format!("{base}/a.txt"), &format!("{base}/a_copy.txt")).unwrap();
    println!("{}", exists(&format!("{base}/a_copy.txt")));

    move_file(
        &format!("{base}/a_copy.txt"),
        &format!("{base}/a_moved.txt"),
    )
    .unwrap();
    println!("{}", exists(&format!("{base}/a_moved.txt")));
    println!("{}", exists(&format!("{base}/a_copy.txt")));

    fs::create_dir_all(format!("{base}/sub")).unwrap();
    write_text(&format!("{base}/sub/nested.txt"), "nested").unwrap();
    rmtree(&format!("{base}/sub")).unwrap();
    println!("{}", exists(&format!("{base}/sub")));

    println!("=== tomllib ===");
    println!("{}", loads("key = \"value\"").unwrap().get("key").is_some());

    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("rm -rf {base}"))
        .status();
    println!("=== Done ===");
}
