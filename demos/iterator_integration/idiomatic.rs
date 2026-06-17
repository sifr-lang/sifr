use regex::Regex;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Clone, Debug)]
struct Match {
    start: i64,
}

impl Match {
    fn start(&self) -> i64 {
        self.start
    }
}

fn adapt_to_iterable<I>(it: I) -> I
where
    I: Iterator<Item = i32>,
{
    it
}

fn collect_starts<I>(it: I) -> Vec<i64>
where
    I: Iterator<Item = Match>,
{
    it.map(|matched| matched.start()).collect()
}

struct MatchIter<'a> {
    regex: Regex,
    text: &'a str,
    cursor: usize,
}

impl Iterator for MatchIter<'_> {
    type Item = Match;

    fn next(&mut self) -> Option<Self::Item> {
        let matched = self.regex.find_at(self.text, self.cursor)?;
        self.cursor = matched.end();
        Some(Match {
            start: matched.start() as i64,
        })
    }
}

fn finditer<'a>(pattern: &str, text: &'a str) -> Result<MatchIter<'a>, regex::Error> {
    Ok(MatchIter {
        regex: Regex::new(pattern)?,
        text,
        cursor: 0,
    })
}

fn write_text(path: &Path, content: &str) -> io::Result<()> {
    fs::write(path, content)
}

fn iterdir(path: &Path) -> io::Result<impl Iterator<Item = io::Result<String>>> {
    Ok(fs::read_dir(path)?.map(|entry| entry.map(|entry| entry.path().display().to_string())))
}

struct RecursiveGlob {
    stack: Vec<fs::ReadDir>,
    suffix: String,
}

impl Iterator for RecursiveGlob {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(read_dir) = self.stack.last_mut() {
            match read_dir.next() {
                Some(Ok(entry)) => {
                    let path = entry.path();
                    if path.is_dir() {
                        match fs::read_dir(&path) {
                            Ok(child_dir) => self.stack.push(child_dir),
                            Err(error) => return Some(Err(error)),
                        }
                    } else if path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.ends_with(&self.suffix))
                    {
                        return Some(Ok(path.display().to_string()));
                    }
                }
                Some(Err(error)) => return Some(Err(error)),
                None => {
                    self.stack.pop();
                }
            }
        }
        None
    }
}

fn rglob(path: &Path, suffix: &str) -> io::Result<RecursiveGlob> {
    Ok(RecursiveGlob {
        stack: vec![fs::read_dir(path)?],
        suffix: suffix.to_string(),
    })
}

fn main() -> io::Result<()> {
    let nums = vec![3, 4, 5].into_iter();
    let via_binding = nums;
    println!("{:?}", via_binding.collect::<Vec<_>>());

    let via_return = adapt_to_iterable(vec![7, 8].into_iter());
    println!("{:?}", via_return.collect::<Vec<_>>());

    let mapped_bytes = b"AZ"
        .iter()
        .copied()
        .map(|byte| byte + 1)
        .collect::<Vec<_>>();
    println!("{mapped_bytes:?}");

    match finditer(r"\d+", "x11y222") {
        Ok(matches) => println!("{:?}", collect_starts(matches)),
        Err(error) => println!("{error}"),
    }

    let base = std::env::temp_dir().join(format!(
        "sifr_iterator_integration{}",
        std::process::id()
    ));
    let nested = base.join("nested");

    fs::create_dir_all(&nested)?;
    write_text(&base.join("a.txt"), "a")?;
    write_text(&nested.join("b.txt"), "b")?;

    let result = (|| -> io::Result<()> {
        let entry_count = iterdir(&base)?
            .take(2)
            .collect::<io::Result<Vec<_>>>()?
            .len();
        println!("{entry_count}");

        let recursive_count = rglob(&base, ".txt")?.collect::<io::Result<Vec<_>>>()?.len();
        println!("{recursive_count}");
        Ok(())
    })();

    let cleanup_result = fs::remove_dir_all(&base);
    result?;
    cleanup_result?;
    Ok(())
}
