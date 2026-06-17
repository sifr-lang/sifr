use regex::Regex;
use std::fmt::{self, Display};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct RegexError {
    message: String,
}

impl Display for RegexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for RegexError {}

impl From<regex::Error> for RegexError {
    fn from(error: regex::Error) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
struct IOError {
    message: String,
}

impl Display for IOError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for IOError {}

impl From<std::io::Error> for IOError {
    fn from(error: std::io::Error) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
struct Match {
    text: String,
}

impl Match {
    fn group(&self) -> &str {
        &self.text
    }
}

fn finditer(pattern: &str, text: &str) -> Result<std::vec::IntoIter<Match>, RegexError> {
    Ok(Regex::new(pattern)?
        .find_iter(text)
        .map(|matched| Match {
            text: matched.as_str().to_string(),
        })
        .collect::<Vec<_>>()
        .into_iter())
}

struct CompiledPattern {
    regex: Regex,
}

impl CompiledPattern {
    fn finditer(&self, text: &str) -> std::vec::IntoIter<Match> {
        self.regex
            .find_iter(text)
            .map(|matched| Match {
                text: matched.as_str().to_string(),
            })
            .collect::<Vec<_>>()
            .into_iter()
    }
}

fn compile(pattern: &str) -> Result<CompiledPattern, RegexError> {
    Ok(CompiledPattern {
        regex: Regex::new(pattern)?,
    })
}

fn wildcard_match(pattern: &str, text: &str) -> bool {
    let pattern = pattern.as_bytes();
    let text = text.as_bytes();
    let mut dp = vec![vec![false; text.len() + 1]; pattern.len() + 1];
    dp[0][0] = true;

    for i in 0..pattern.len() {
        if pattern[i] == b'*' {
            dp[i + 1][0] = dp[i][0];
        }
        for j in 0..text.len() {
            dp[i + 1][j + 1] = match pattern[i] {
                b'*' => dp[i][j + 1] || dp[i + 1][j],
                b'?' => dp[i][j],
                ch => dp[i][j] && ch == text[j],
            };
        }
    }

    dp[pattern.len()][text.len()]
}

fn sorted_dir_entries(dir: &Path) -> Result<Vec<PathBuf>, IOError> {
    let mut entries = fs::read_dir(dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort();
    Ok(entries)
}

fn iglob(dir: &Path, pattern: &str) -> Result<std::vec::IntoIter<String>, IOError> {
    let names = sorted_dir_entries(dir)?
        .into_iter()
        .filter(|path| path.is_file())
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?.to_string();
            wildcard_match(pattern, &name).then_some(name)
        })
        .collect::<Vec<_>>();
    Ok(names.into_iter())
}

#[derive(Debug, Clone)]
struct DemoPath {
    path: PathBuf,
}

impl DemoPath {
    fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    fn iterdir(&self) -> Result<std::vec::IntoIter<String>, IOError> {
        let entries = sorted_dir_entries(&self.path)?
            .into_iter()
            .filter_map(|path| path.to_str().map(str::to_string))
            .collect::<Vec<_>>();
        Ok(entries.into_iter())
    }

    fn glob(&self, pattern: &str) -> Result<std::vec::IntoIter<String>, IOError> {
        let matches = sorted_dir_entries(&self.path)?
            .into_iter()
            .filter(|path| path.is_file())
            .filter_map(|path| {
                let name = path.file_name()?.to_str()?;
                wildcard_match(pattern, name).then(|| path.to_string_lossy().to_string())
            })
            .collect::<Vec<_>>();
        Ok(matches.into_iter())
    }

    fn rglob(&self, pattern: &str) -> Result<std::vec::IntoIter<String>, IOError> {
        fn walk(dir: &Path, pattern: &str, out: &mut Vec<String>) -> Result<(), IOError> {
            for entry in sorted_dir_entries(dir)? {
                if entry.is_dir() {
                    walk(&entry, pattern, out)?;
                } else if entry.is_file()
                    && entry
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| wildcard_match(pattern, name))
                {
                    out.push(entry.to_string_lossy().to_string());
                }
            }
            Ok(())
        }

        let mut matches = Vec::new();
        walk(&self.path, pattern, &mut matches)?;
        Ok(matches.into_iter())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    (|| -> Result<(), RegexError> {
        let mut digits = finditer("\\d+", "v1 and v22")?;
        let first = digits.next();
        let second = digits.next();
        assert_eq!(first.as_ref().map(Match::group), Some("1"));
        assert_eq!(second.as_ref().map(Match::group), Some("22"));
        assert!(digits.next().is_none());

        let pattern = compile("[a-z]+")?;
        let words = pattern
            .finditer("alpha 123 beta")
            .map(|matched| matched.group().to_string())
            .collect::<Vec<_>>();
        assert_eq!(format!("{words:?}"), "[\"alpha\", \"beta\"]");
        Ok(())
    })()?;

    let base = PathBuf::from(format!(
        "/tmp/sifr_regex_filesystem_demo_{}",
        std::process::id()
    ));
    let fs_result = (|| -> Result<(), IOError> {
        if base.exists() {
            fs::remove_dir_all(&base)?;
        }
        fs::create_dir_all(base.join("sub"))?;
        fs::write(base.join("a.txt"), "a")?;
        fs::write(base.join("sub/b.txt"), "b")?;

        let iglobbed = iglob(&base, "*.txt")?.collect::<Vec<_>>();
        assert_eq!(format!("{iglobbed:?}"), "[\"a.txt\"]");

        let root = DemoPath::new(base.clone());
        let entries = root.iterdir()?.collect::<Vec<_>>();
        assert!(entries.len() >= 2);

        let top_txt = root.glob("*.txt")?.collect::<Vec<_>>();
        assert_eq!(
            top_txt,
            vec![base.join("a.txt").to_string_lossy().to_string()]
        );

        let recursive_txt = root.rglob("*.txt")?.collect::<Vec<_>>();
        assert_eq!(recursive_txt.len(), 2);
        Ok(())
    })();

    if base.exists() {
        fs::remove_dir_all(&base)?;
    }

    fs_result?;
    println!("parity_ext_regex_and_filesystem_filesystem_iterators_demo: ok");
    Ok(())
}
