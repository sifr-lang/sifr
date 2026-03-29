use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use regex::Regex;
use serde_json::Value;

fn b64encode(text: &str) -> String {
    STANDARD.encode(text.as_bytes())
}

fn b64decode(text: &str) -> Result<String, String> {
    let bytes = STANDARD.decode(text).map_err(|error| error.to_string())?;
    String::from_utf8(bytes).map_err(|error| error.to_string())
}

#[derive(Clone)]
struct Rng {
    state: u64,
}

impl Rng {
    fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_nanos() as u64;
        Self { state: seed | 1 }
    }

    fn next_u64(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    fn random(&mut self) -> f64 {
        self.next_u64() as f64 / (u64::MAX as f64 + 1.0)
    }

    fn randint(&mut self, low: i64, high: i64) -> Result<i64, String> {
        if high < low {
            return Err("empty range for randint()".to_string());
        }
        let span = (high - low + 1) as u64;
        Ok(low + (self.next_u64() % span) as i64)
    }

    fn uniform(&mut self, low: f64, high: f64) -> f64 {
        low + (high - low) * self.random()
    }
}

fn fnmatch_filter<'a>(names: &'a [&'a str], pattern: &str) -> Vec<&'a str> {
    match pattern {
        "*.py" => names
            .iter()
            .copied()
            .filter(|name| name.ends_with(".py"))
            .collect(),
        _ => names
            .iter()
            .copied()
            .filter(|name| *name == pattern)
            .collect(),
    }
}

fn search(pattern: &str, text: &str) -> Result<Option<String>, String> {
    let regex = Regex::new(pattern).map_err(|error| error.to_string())?;
    Ok(regex.find(text).map(|value| value.as_str().to_string()))
}

fn findall(pattern: &str, text: &str) -> Result<Vec<String>, String> {
    let regex = Regex::new(pattern).map_err(|error| error.to_string())?;
    Ok(regex
        .find_iter(text)
        .map(|value| value.as_str().to_string())
        .collect())
}

fn split(pattern: &str, text: &str) -> Result<Vec<String>, String> {
    let regex = Regex::new(pattern).map_err(|error| error.to_string())?;
    Ok(regex.split(text).map(str::to_string).collect())
}

fn sub(pattern: &str, replacement: &str, text: &str) -> Result<String, String> {
    let regex = Regex::new(pattern).map_err(|error| error.to_string())?;
    Ok(regex.replace_all(text, replacement).into_owned())
}

fn loads(text: &str) -> Result<Value, String> {
    serde_json::from_str(text).map_err(|error| error.to_string())
}

fn json_dumps(text: &str) -> Result<String, String> {
    serde_json::to_string(text).map_err(|error| error.to_string())
}

fn capwords(text: &str) -> String {
    text.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first
                    .to_uppercase()
                    .chain(chars.flat_map(|ch| ch.to_lowercase()))
                    .collect(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn time_now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs_f64()
}

fn main() {
    println!("{}", 2.0_f64.powf(10.0));
    println!("{}", (-42.5_f64).abs());

    let encoded = b64encode("Hello, Sifr!");
    println!("{encoded}");
    match b64decode(&encoded) {
        Ok(decoded) => println!("{decoded}"),
        Err(message) => println!("base64 error: {message}"),
    }

    let mut rng = Rng::new();
    match rng.randint(1, 100) {
        Ok(value) => {
            println!("{}", value >= 1);
            println!("{}", value <= 100);
        }
        Err(message) => println!("error: {message}"),
    }
    let random_value = rng.random();
    println!("{}", random_value >= 0.0);
    println!("{}", random_value < 1.0);
    let uniform_value = rng.uniform(10.0, 20.0);
    println!("{}", uniform_value >= 10.0);
    println!("{}", uniform_value <= 20.0);

    println!("{}", !std::env::consts::OS.is_empty());
    println!("{}", !std::env::consts::ARCH.is_empty());

    println!("{}", time_now() > 0.0);

    let names = ["foo.py", "bar.txt", "baz.py", "qux.rs"];
    println!("{}", fnmatch_filter(&names, "*.py").len());

    match search("[0-9]+", "hello 42 world") {
        Ok(Some(found)) => println!("{found}"),
        Ok(None) => {}
        Err(message) => println!("regex error: {message}"),
    }
    match findall("[0-9]+", "abc123def456ghi789") {
        Ok(all_nums) => println!("{}", all_nums.len()),
        Err(message) => println!("regex error: {message}"),
    }
    match split(",", "a,b,c,d") {
        Ok(parts) => println!("{}", parts.len()),
        Err(message) => println!("regex error: {message}"),
    }
    match sub("[0-9]+", "NUM", "item1 and item2") {
        Ok(replaced) => println!("{replaced}"),
        Err(message) => println!("regex error: {message}"),
    }

    match loads("{\"key\":\"value\"}") {
        Ok(data) => println!("{data}"),
        Err(message) => println!("json error: {message}"),
    }
    match json_dumps("hello") {
        Ok(output) => println!("{output}"),
        Err(message) => println!("json error: {message}"),
    }

    println!("{}", capwords("hello world from sifr"));
}
