use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use sha2::{Digest, Sha256};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
struct ParseError(String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone)]
struct ValueError(String);

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for ValueError {}

fn global_seed() -> &'static Mutex<u64> {
    static GLOBAL: OnceLock<Mutex<u64>> = OnceLock::new();
    GLOBAL.get_or_init(|| {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        Mutex::new(seed)
    })
}

fn lock_global_seed() -> MutexGuard<'static, u64> {
    match global_seed().lock() {
        Ok(state) => state,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn next_u64(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

fn randint(low: i64, high: i64) -> Result<i64, ValueError> {
    if low > high {
        return Err(ValueError(
            "random.randint: low must be <= high".to_string(),
        ));
    }
    let mut state = lock_global_seed();
    let span = (high - low + 1) as u64;
    Ok(low + (next_u64(&mut state) % span) as i64)
}

fn b64encode(text: &str) -> String {
    STANDARD.encode(text)
}

fn b64decode(text: &str) -> Result<String, ParseError> {
    let bytes = STANDARD
        .decode(text)
        .map_err(|error| ParseError(error.to_string()))?;
    String::from_utf8(bytes).map_err(|error| ParseError(error.to_string()))
}

#[derive(Debug, Clone)]
struct Sha256Object {
    input: Vec<u8>,
}

impl Sha256Object {
    fn hexdigest(&self) -> String {
        let digest = Sha256::digest(&self.input);
        format!("{digest:x}")
    }
}

fn sha256(data: &[u8]) -> Sha256Object {
    Sha256Object {
        input: data.to_vec(),
    }
}

fn main() {
    let mut range_ok = false;
    if let Ok(value) = randint(1, 5) {
        range_ok = (1..=5).contains(&value);
    }
    assert!(range_ok);

    let payload = "random_hashing_seed";
    let encoded = b64encode(payload);
    let decode_ok = b64decode(&encoded).is_ok_and(|decoded| decoded == payload);
    assert!(decode_ok);

    let digest = sha256(b"random_hashing_seed").hexdigest();
    assert_eq!(digest.len(), 64);

    println!("rng_random_hashing_lock_demo: pass");
}
