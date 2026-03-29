use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct Counter {
    counts: BTreeMap<String, i64>,
}

impl Counter {
    fn from_list(values: &[&str]) -> Self {
        let mut counter = Self::default();
        for value in values {
            *counter.counts.entry((*value).to_string()).or_insert(0) += 1;
        }
        counter
    }

    fn get(&self, key: &str) -> i64 {
        self.counts.get(key).copied().unwrap_or(0)
    }
}

fn set_from_list(values: &[i64]) -> BTreeSet<i64> {
    values.iter().copied().collect()
}

fn set_add(mut values: BTreeSet<i64>, value: i64) -> BTreeSet<i64> {
    values.insert(value);
    values
}

fn set_contains(values: &BTreeSet<i64>, value: i64) -> bool {
    values.contains(&value)
}

fn set_union(left: &BTreeSet<i64>, right: &BTreeSet<i64>) -> BTreeSet<i64> {
    left.union(right).copied().collect()
}

fn set_intersection(left: &BTreeSet<i64>, right: &BTreeSet<i64>) -> BTreeSet<i64> {
    left.intersection(right).copied().collect()
}

fn encode_utf8(text: &str) -> Vec<u8> {
    text.as_bytes().to_vec()
}

fn decode_utf8(bytes: &[u8]) -> Result<String, std::string::FromUtf8Error> {
    String::from_utf8(bytes.to_vec())
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn bytes_from_hex(text: &str) -> Result<Vec<u8>, String> {
    if text.len() % 2 != 0 {
        return Err("fromhex() arg must contain an even number of hexadecimal digits".to_string());
    }

    text.as_bytes()
        .chunks(2)
        .map(|chunk| {
            let pair = std::str::from_utf8(chunk).map_err(|err| err.to_string())?;
            u8::from_str_radix(pair, 16).map_err(|err| err.to_string())
        })
        .collect()
}

fn main() {
    println!("=== Set Operations ===");
    println!(
        "Set from [1,2,2,3,3]: length = {}",
        set_from_list(&[1, 2, 2, 3, 3]).len()
    );
    println!(
        "After adding 4: length = {}",
        set_add(set_from_list(&[1, 2, 3]), 4).len()
    );
    println!(
        "Contains 2: {}",
        set_contains(&set_from_list(&[1, 2, 3]), 2)
    );
    println!(
        "Contains 5: {}",
        set_contains(&set_from_list(&[1, 2, 3]), 5)
    );
    println!(
        "Union [1,2,3] | [3,4,5]: length = {}",
        set_union(&set_from_list(&[1, 2, 3]), &set_from_list(&[3, 4, 5])).len()
    );
    println!(
        "Intersection [1,2,3] & [3,4,5]: length = {}",
        set_intersection(&set_from_list(&[1, 2, 3]), &set_from_list(&[3, 4, 5])).len()
    );

    println!("=== Counter ===");
    let fruit_counter =
        Counter::from_list(&["apple", "banana", "apple", "cherry", "banana", "apple"]);
    println!("apple count: {}", fruit_counter.get("apple"));
    println!("banana count: {}", fruit_counter.get("banana"));
    println!("cherry count: {}", fruit_counter.get("cherry"));

    println!("=== Bytes ===");
    println!("'hello' encoded: {} bytes", encode_utf8("hello").len());
    match decode_utf8(&encode_utf8("Sifr")) {
        Ok(roundtrip) => println!("Roundtrip: {roundtrip}"),
        Err(err) => println!("decode error: {err}"),
    }
    println!("'hello' as hex: {}", bytes_to_hex(&encode_utf8("hello")));
    match bytes_from_hex("536966")
        .and_then(|bytes| decode_utf8(&bytes).map_err(|err| err.to_string()))
    {
        Ok(decoded) => println!("Hex '536966' decoded: {decoded}"),
        Err(err) => println!("hex error: {err}"),
    }

    println!("=== Demo complete ===");
}
