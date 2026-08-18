use sha2::{Digest, Sha256};
use std::fs;

#[derive(Debug, Clone)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ValueError {}

#[derive(Debug, Clone)]
struct HashlibError {
    message: String,
}

impl HashlibError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for HashlibError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for HashlibError {}

fn assert_vector_eq(actual: &[String], expected: &[String]) {
    assert_eq!(actual, expected);
}

fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(actual, expected);
}

fn encode_md5(data: &[u8]) -> [u8; 16] {
    md5::compute(data).0
}

fn encode_sha256(data: &[u8]) -> [u8; 32] {
    Sha256::digest(data).into()
}

fn hexify(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn bytes_for_algorithm(name: &str, data: &[u8]) -> Result<Vec<u8>, ValueError> {
    match name {
        "md5" => Ok(encode_md5(data).to_vec()),
        "sha256" => Ok(encode_sha256(data).to_vec()),
        _ => Err(ValueError::new(format!(
            "unsupported hash algorithm: {name}"
        ))),
    }
}

#[derive(Clone, Debug)]
struct HashObject {
    name: String,
    data: Vec<u8>,
}

impl HashObject {
    fn update(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }

    fn hexdigest(&self) -> String {
        bytes_for_algorithm(&self.name, &self.data)
            .map(|bytes| hexify(&bytes))
            .unwrap_or_default()
    }

    fn digest(&self) -> Vec<u8> {
        bytes_for_algorithm(&self.name, &self.data).unwrap_or_default()
    }
}

fn sha256(data: &[u8]) -> HashObject {
    HashObject {
        name: "sha256".to_string(),
        data: data.to_vec(),
    }
}

fn md5(data: &[u8]) -> HashObject {
    HashObject {
        name: "md5".to_string(),
        data: data.to_vec(),
    }
}

fn copy_hash(hash: &HashObject) -> HashObject {
    hash.clone()
}

fn new(name: &str, data: &[u8]) -> Result<HashObject, ValueError> {
    match name {
        "md5" => Ok(md5(data)),
        "sha256" => Ok(sha256(data)),
        _ => Err(ValueError::new(format!(
            "unsupported hash algorithm: {name}"
        ))),
    }
}

fn algorithms_guaranteed() -> Vec<String> {
    vec!["md5".to_string(), "sha256".to_string()]
}

fn file_digest(path: &str, name: &str) -> Result<String, HashlibError> {
    let data = fs::read(path).map_err(|error| HashlibError::new(error.to_string()))?;
    bytes_for_algorithm(name, &data)
        .map(|bytes| hexify(&bytes))
        .map_err(|error| HashlibError::new(error.message))
}

fn write_text(path: &str, content: &str) -> Result<(), std::io::Error> {
    fs::write(path, content)
}

fn contains(values: &[String], needle: &str) -> bool {
    values.iter().any(|value| value == needle)
}

fn collect_positive_actual(tmp_path: &str) -> Vec<String> {
    let mut actual = Vec::new();

    let mut hash = sha256(b"");
    hash.update(b"a");
    hash.update(b"bc");
    actual.push((hash.hexdigest() == sha256(b"abc").hexdigest()).to_string());
    actual.push((hash.digest().len() == 32).to_string());

    let mut copy = copy_hash(&hash);
    copy.update(b"x");
    actual.push((copy.hexdigest() == sha256(b"abcx").hexdigest()).to_string());

    let md5_hash = md5(b"hello");
    actual.push((md5_hash.hexdigest() == md5(b"hello").hexdigest()).to_string());
    actual.push(contains(&algorithms_guaranteed(), "sha256").to_string());
    actual.push(md5_hash.hexdigest());
    actual.push(file_digest(tmp_path, "sha256").unwrap_or_else(|_| "ERR".to_string()));

    actual
}

fn collect_negative_actual_ok() -> Vec<bool> {
    vec![new("sha3_256", b"").is_ok()]
}

fn main() {
    let tmp_path = "tmp_hashlib_hashlib_demo.txt";
    let _ = write_text(tmp_path, "file-data");

    assert_vector_eq(
        &collect_positive_actual(tmp_path),
        &[
            "true".to_string(),
            "true".to_string(),
            "true".to_string(),
            "true".to_string(),
            "true".to_string(),
            "5d41402abc4b2a76b9719d911017c592".to_string(),
            "8e6537b695ff181bc341e32d8b8970485ac3513408e5eb1e8ba9fc5af1cd3f57".to_string(),
        ],
    );
    assert_bool_vector_eq(&collect_negative_actual_ok(), &[false]);

    println!("hashlib hashlib parity demo: pass");
}
