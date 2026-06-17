use std::collections::HashMap;
use std::fs;

fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(actual, expected);
}

fn glob(directory: &str, pattern: &str) -> std::io::Result<Vec<String>> {
    let read_dir = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };

    let include_hidden = pattern.starts_with('.');
    let mut matches = Vec::new();

    for entry in read_dir.flatten() {
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

fn collect_glob_actual() -> Vec<bool> {
    let base = format!("/tmp/sifr_glob_glob_demo_{}", std::process::id());
    let _ = fs::remove_dir_all(&base);

    let result = (|| -> std::io::Result<Vec<bool>> {
        fs::create_dir_all(&base)?;
        fs::write(format!("{base}/a.txt"), "a")?;
        fs::write(format!("{base}/b.txt"), "b")?;
        fs::write(format!("{base}/.hidden.txt"), "h")?;

        let txt = glob(&base, "*.txt")?;
        let hidden = glob(&base, ".*.txt")?;
        let wildcard_q = glob(&base, "?.txt")?;
        let none = glob(&base, "*.csv")?;
        let missing = glob(&format!("{base}_missing"), "*.txt")?;

        Ok(vec![
            txt == ["a.txt".to_string(), "b.txt".to_string()],
            hidden == [".hidden.txt".to_string()],
            wildcard_q == ["a.txt".to_string(), "b.txt".to_string()],
            none.is_empty(),
            missing.is_empty(),
        ])
    })();

    let _ = fs::remove_dir_all(&base);
    result.unwrap_or_else(|_| vec![false, false, false, false, false])
}

fn main() {
    assert_bool_vector_eq(&collect_glob_actual(), &[true, true, true, true, true]);
    println!("glob glob parity demo: pass");
}
