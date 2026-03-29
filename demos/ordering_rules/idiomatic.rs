use serde_json::Value;

struct Processor {
    name: String,
    transform: fn(i64) -> i64,
}

impl Processor {
    fn new(name: &str, transform: fn(i64) -> i64) -> Self {
        Self {
            name: name.to_string(),
            transform,
        }
    }
}

fn triple(x: i64) -> i64 {
    x * 3
}

fn repeat<T: Clone>(value: T, times: usize) -> impl Iterator<Item = T> {
    std::iter::repeat_n(value, times)
}

fn chain<T>(
    left: impl IntoIterator<Item = T>,
    right: impl IntoIterator<Item = T>,
) -> impl Iterator<Item = T> {
    left.into_iter().chain(right)
}

fn take<T>(n: usize, values: impl IntoIterator<Item = T>) -> Vec<T> {
    values.into_iter().take(n).collect()
}

fn search(pattern: &str, text: &str) -> Result<Option<String>, regex::Error> {
    Ok(regex::Regex::new(pattern)?
        .find(text)
        .map(|matched| matched.as_str().to_string()))
}

fn sub(pattern: &str, replacement: &str, text: &str) -> Result<String, regex::Error> {
    Ok(regex::Regex::new(pattern)?
        .replace_all(text, replacement)
        .into_owned())
}

fn findall(pattern: &str, text: &str) -> Result<Vec<String>, regex::Error> {
    Ok(regex::Regex::new(pattern)?
        .find_iter(text)
        .map(|matched| matched.as_str().to_string())
        .collect())
}

fn split(pattern: &str, text: &str) -> Result<Vec<String>, regex::Error> {
    Ok(regex::Regex::new(pattern)?
        .split(text)
        .map(str::to_string)
        .collect())
}

fn main() {
    let repeated = repeat(7_i64, 3).collect::<Vec<_>>();
    println!("{repeated:?}");

    for value in repeat(42_i64, 2) {
        println!("{value}");
    }

    let chained = chain([1_i64, 2], [3, 4]).collect::<Vec<_>>();
    println!("{chained:?}");

    let first_two = take(2, [10_i64, 20, 30, 40]);
    println!("{first_two:?}");

    let parsed: Value = serde_json::from_str("{\"key\": 42}").expect("literal json should parse");
    println!("{parsed}");

    let found = search("\\d+", "abc123").expect("literal regex should compile");
    if let Some(found) = found {
        println!("{found}");
    }

    println!(
        "{}",
        sub("\\d", "X", "a1b2c3").expect("literal regex should compile")
    );
    println!(
        "{:?}",
        findall("\\w+", "hello world").expect("literal regex should compile")
    );
    println!(
        "{:?}",
        split(",", "a,b,c").expect("literal regex should compile")
    );

    let processor = Processor::new("tripler", triple);
    println!("{}", processor.name);
    println!("{}", (processor.transform)(10));
}
