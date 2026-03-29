use blake2::{Blake2b512, Blake2s256, Digest};
use chrono::{Datelike, Local, NaiveDateTime, TimeZone, Utc};
use data_encoding::BASE32;
use fs2::total_space;
use libm::{erf, erfc, frexp, ldexp, lgamma, modf, tgamma};
use sha2::{Sha224, Sha384};

#[derive(Clone, Copy)]
struct StructTime {
    tm_year: i32,
    tm_yday: u32,
}

fn nextafter(from: f64, to: f64) -> f64 {
    if from == to {
        from
    } else if from < to {
        from.next_up()
    } else {
        from.next_down()
    }
}

fn ulp(value: f64) -> f64 {
    nextafter(value, f64::INFINITY) - value
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn sha224_hex(text: &str) -> String {
    let mut hasher = Sha224::new();
    hasher.update(text.as_bytes());
    hex_digest(&hasher.finalize())
}

fn sha384_hex(text: &str) -> String {
    let mut hasher = Sha384::new();
    hasher.update(text.as_bytes());
    hex_digest(&hasher.finalize())
}

fn blake2b_hex(text: &str) -> String {
    let mut hasher = Blake2b512::new();
    hasher.update(text.as_bytes());
    hex_digest(&hasher.finalize())
}

fn blake2s_hex(text: &str) -> String {
    let mut hasher = Blake2s256::new();
    hasher.update(text.as_bytes());
    hex_digest(&hasher.finalize())
}

fn system() -> &'static str {
    std::env::consts::OS
}

fn machine() -> &'static str {
    std::env::consts::ARCH
}

fn processor() -> &'static str {
    std::env::consts::ARCH
}

fn gmtime_struct(seconds: i64) -> StructTime {
    let timestamp = Utc.timestamp_opt(seconds, 0).single().unwrap();
    StructTime {
        tm_year: timestamp.year(),
        tm_yday: timestamp.ordinal(),
    }
}

fn localtime_struct(seconds: i64) -> StructTime {
    let timestamp = Local.timestamp_opt(seconds, 0).single().unwrap();
    StructTime {
        tm_year: timestamp.year(),
        tm_yday: timestamp.ordinal(),
    }
}

fn strptime(input: &str, format: &str) -> Result<String, chrono::ParseError> {
    Ok(NaiveDateTime::parse_from_str(input, format)?
        .format("%Y-%m-%d %H:%M:%S")
        .to_string())
}

fn b32encode(text: &str) -> String {
    BASE32.encode(text.as_bytes())
}

fn b32decode(text: &str) -> Result<String, String> {
    let bytes = BASE32
        .decode(text.as_bytes())
        .map_err(|err| err.to_string())?;
    String::from_utf8(bytes).map_err(|err| err.to_string())
}

fn demo_math() {
    println!("=== math new intrinsics ===");
    let e0 = erf(0.0);
    println!("erf near 0 = {}", e0 < 0.001 && e0 > -0.001);
    let ec0 = erfc(0.0);
    println!("erfc near 1 = {}", ec0 > 0.999 && ec0 < 1.001);
    let gamma_5 = tgamma(5.0);
    println!("gamma(5) > 23 = {}", gamma_5 > 23.0);
    let log_gamma_5 = lgamma(5.0);
    println!("lgamma(5) > 3 = {}", log_gamma_5 > 3.0);
    let (mantissa, _) = frexp(8.0);
    println!("frexp(8.0) mantissa = {mantissa}");
    println!("ldexp(0.5, 4) = {}", ldexp(0.5, 4));
    let (fractional, _) = modf(3.7);
    println!("modf(3.7) frac > 0 = {}", fractional > 0.0);
    println!("nextafter(1.0, 2.0) > 1.0 = {}", nextafter(1.0, 2.0) > 1.0);
    println!("ulp(1.0) > 0 = {}", ulp(1.0) > 0.0);
}

fn demo_os() {
    println!("=== os new intrinsics ===");
    println!("pid > 0 = {}", std::process::id() > 0);
    let cpu_count = std::thread::available_parallelism()
        .map(|value| value.get())
        .unwrap_or(1);
    println!("cpu_count >= 1 = {}", cpu_count >= 1);
}

fn demo_hashlib() {
    println!("=== hashlib new intrinsics ===");
    let text = "hello";
    println!("sha224 len = {}", sha224_hex(text).len());
    println!("sha384 len = {}", sha384_hex(text).len());
    println!("blake2b len = {}", blake2b_hex(text).len());
    println!("blake2s len = {}", blake2s_hex(text).len());
}

fn demo_platform() {
    println!("=== platform new intrinsics ===");
    println!("system len > 0 = {}", !system().is_empty());
    println!("machine len > 0 = {}", !machine().is_empty());
    println!("processor len > 0 = {}", !processor().is_empty());
}

fn demo_time() {
    println!("=== time new intrinsics ===");
    println!("gmtime year = {}", gmtime_struct(0).tm_year == 1970);
    println!("localtime yday >= 1 = {}", localtime_struct(0).tm_yday >= 1);
    match strptime("2024-01-15 10:30:00", "%Y-%m-%d %H:%M:%S") {
        Ok(parsed) => println!("strptime ok = {}", !parsed.is_empty()),
        Err(err) => println!("strptime error: {err}"),
    }
}

fn demo_base64() {
    println!("=== base64 new intrinsics ===");
    let encoded = b32encode("hello world");
    println!("b32encode len > 0 = {}", !encoded.is_empty());
    match b32decode(&encoded) {
        Ok(decoded) => println!("b32decode = {decoded}"),
        Err(err) => println!("b32decode error: {err}"),
    }
}

fn demo_shutil() {
    println!("=== shutil new intrinsics ===");
    if let Ok(disk_total) = total_space("/") {
        println!("disk_total > 0 = {}", disk_total > 0);
    }
}

fn main() {
    demo_math();
    demo_os();
    demo_hashlib();
    demo_platform();
    demo_time();
    demo_base64();
    demo_shutil();
}
