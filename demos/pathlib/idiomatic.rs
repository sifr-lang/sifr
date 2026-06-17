use std::collections::HashMap;
use std::fs;
use std::path::{Path as StdPath, PathBuf};

fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(actual, expected);
}

type IOError = std::io::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Path {
    raw: PathBuf,
}

impl Path {
    fn new(path: impl AsRef<StdPath>) -> Self {
        Self {
            raw: path.as_ref().to_path_buf(),
        }
    }

    fn exists(&self) -> bool {
        self.raw.exists()
    }

    fn is_file(&self) -> bool {
        self.raw.is_file()
    }

    fn is_dir(&self) -> bool {
        self.raw.is_dir()
    }

    fn read_text(&self) -> Result<String, IOError> {
        fs::read_to_string(&self.raw)
    }

    fn write_text(&self, content: &str) -> Result<(), IOError> {
        fs::write(&self.raw, content)
    }

    fn mkdir(&self) -> Result<(), IOError> {
        fs::create_dir_all(&self.raw)
    }

    fn unlink(&self) -> Result<(), IOError> {
        fs::remove_file(&self.raw)
    }

    fn rmdir(&self) -> Result<(), IOError> {
        fs::remove_dir(&self.raw)
    }

    fn glob(&self, pattern: &str) -> Result<Vec<String>, IOError> {
        glob_entries(&self.raw, pattern)
    }
}

fn join_path(base: &str, child: &str) -> String {
    StdPath::new(base)
        .join(child)
        .to_string_lossy()
        .into_owned()
}

fn basename(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}

fn glob_entries(directory: &StdPath, pattern: &str) -> Result<Vec<String>, IOError> {
    let read_dir = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };

    let include_hidden = pattern.starts_with('.');
    let mut matches = Vec::new();

    for entry in read_dir {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();

        if !include_hidden && name.starts_with('.') {
            continue;
        }
        if wildcard_match(&name, pattern) {
            matches.push(name);
        }
    }

    matches.sort();
    Ok(matches)
}

fn wildcard_match(name: &str, pattern: &str) -> bool {
    fn inner(
        name: &[char],
        pattern: &[char],
        ni: usize,
        pi: usize,
        memo: &mut HashMap<(usize, usize), bool>,
    ) -> bool {
        if let Some(result) = memo.get(&(ni, pi)) {
            return *result;
        }

        let result = match pattern.get(pi) {
            None => ni == name.len(),
            Some('*') => (ni..=name.len()).any(|next| inner(name, pattern, next, pi + 1, memo)),
            Some('?') => ni < name.len() && inner(name, pattern, ni + 1, pi + 1, memo),
            Some(expected) => {
                ni < name.len()
                    && name[ni] == *expected
                    && inner(name, pattern, ni + 1, pi + 1, memo)
            }
        };

        memo.insert((ni, pi), result);
        result
    }

    let mut memo = HashMap::new();
    let name_chars: Vec<char> = name.chars().collect();
    let pattern_chars: Vec<char> = pattern.chars().collect();
    inner(&name_chars, &pattern_chars, 0, 0, &mut memo)
}

fn collect_path_helpers_actual() -> Vec<bool> {
    vec![
        basename("/tmp/demo.txt") == "demo.txt",
        join_path("/tmp", "demo.txt") == "/tmp/demo.txt",
    ]
}

fn collect_path_class_actual() -> Vec<bool> {
    let base = format!("/tmp/sifr_pathlib_pathlib_demo_{}", std::process::id());
    let _ = fs::remove_dir_all(&base);

    let filep = Path::new(join_path(&base, "demo.txt"));
    let dirp = Path::new(base.clone());

    let mut path_flow_ok = false;
    let mut glob_ok = false;
    let mut cleanup_ok = false;

    if dirp.mkdir().is_ok()
        && filep.write_text("hello").is_ok()
        && filep.read_text().ok().as_deref() == Some("hello")
    {
        path_flow_ok = filep.exists() && filep.is_file() && dirp.is_dir();
        glob_ok = dirp
            .glob("*.txt")
            .map(|matches| matches.iter().any(|name| name == "demo.txt"))
            .unwrap_or(false);
        cleanup_ok = filep.unlink().is_ok() && dirp.rmdir().is_ok() && !dirp.exists();
    }

    vec![path_flow_ok, glob_ok, cleanup_ok]
}

fn collect_missing_path_actual() -> Vec<bool> {
    vec![Path::new("/tmp/sifr_pathlib_pathlib_demo_missing.txt")
        .read_text()
        .is_err()]
}

fn main() {
    let mut actual = Vec::new();
    actual.extend(collect_path_helpers_actual());
    actual.extend(collect_path_class_actual());
    actual.extend(collect_missing_path_actual());

    assert_bool_vector_eq(&actual, &[true, true, true, true, true, true]);
    println!("pathlib pathlib parity demo: pass");
}
