// --- stdlib: sifr.hashlib ---
#[derive(Debug, Clone, PartialEq)]
struct HashObject {
    _algorithm: String,
    _data: Vec<u8>,
    name: String,
    digest_size: i64,
    block_size: i64,
}
impl HashObject {
    fn new(
        algorithm: String,
        data: Vec<u8>,
        name: String,
        digest_size: i64,
        block_size: i64,
    ) -> Self {
        return Self {
            _algorithm: algorithm,
            _data: data,
            name: name,
            digest_size: digest_size,
            block_size: block_size,
        };
    }
    fn update(&mut self, data: &String) {
        self._data = {
            let mut __v = (self._data.clone()).clone();
            __v.extend(
                ({
                    let __s = data;
                    __s.as_bytes().to_vec()
                })
                    .iter()
                    .cloned(),
            );
            __v
        };
    }
    fn update_bytes(&mut self, data: &Vec<u8>) {
        self._data = {
            let mut __v = (self._data.clone()).clone();
            __v.extend((data).iter().cloned());
            __v
        };
    }
    fn hexdigest(&self) -> String {
        return _hash_hex(&self._algorithm.clone(), &self._data.clone());
    }
    fn digest(&self) -> Vec<u8> {
        return _hash_bytes(&self._algorithm.clone(), &self._data.clone());
    }
    fn digest_bytes(&self) -> Vec<u8> {
        return self.digest();
    }
}
fn _build_hash(algorithm: &String, data: &Vec<u8>) -> HashObject {
    let alg: String = algorithm.to_lowercase();
    if alg == "md5".to_string() {
        return HashObject::new(
            alg,
            (data).clone(),
            "md5".to_string(),
            16 as i64,
            64 as i64,
        );
    } else {
        if alg == "sha1".to_string() {
            return HashObject::new(
                alg,
                (data).clone(),
                "sha1".to_string(),
                20 as i64,
                64 as i64,
            );
        } else {
            if alg == "sha224".to_string() {
                return HashObject::new(
                    alg,
                    (data).clone(),
                    "sha224".to_string(),
                    28 as i64,
                    64 as i64,
                );
            } else {
                if alg == "sha256".to_string() {
                    return HashObject::new(
                        alg,
                        (data).clone(),
                        "sha256".to_string(),
                        32 as i64,
                        64 as i64,
                    );
                } else {
                    if alg == "sha384".to_string() {
                        return HashObject::new(
                            alg,
                            (data).clone(),
                            "sha384".to_string(),
                            48 as i64,
                            128 as i64,
                        );
                    } else {
                        if alg == "sha512".to_string() {
                            return HashObject::new(
                                alg,
                                (data).clone(),
                                "sha512".to_string(),
                                64 as i64,
                                128 as i64,
                            );
                        } else {
                            if alg == "blake2b".to_string() {
                                return HashObject::new(
                                    alg,
                                    (data).clone(),
                                    "blake2b".to_string(),
                                    64 as i64,
                                    128 as i64,
                                );
                            } else {
                                if alg == "blake2s".to_string() {
                                    return HashObject::new(
                                        alg,
                                        (data).clone(),
                                        "blake2s".to_string(),
                                        32 as i64,
                                        64 as i64,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    return HashObject::new(
        alg,
        (data).clone(),
        "unknown".to_string(),
        0 as i64,
        0 as i64,
    );
}
fn _is_supported_algorithm(name: &String) -> bool {
    let n: String = name.to_lowercase();
    return (((((((n == "md5".to_string()) || (n == "sha1".to_string()))
        || (n == "sha224".to_string())) || (n == "sha256".to_string()))
        || (n == "sha384".to_string())) || (n == "sha512".to_string()))
        || (n == "blake2b".to_string())) || (n == "blake2s".to_string());
}
fn _hash_bytes(algorithm: &String, data: &Vec<u8>) -> Vec<u8> {
    if algorithm.clone() == "md5".to_string() {
        return md5::compute((data)).0.to_vec();
    } else {
        if algorithm.clone() == "sha1".to_string() {
            return (<sha1::Sha1 as sha1::Digest>::digest)((data)).to_vec();
        } else {
            if algorithm.clone() == "sha224".to_string() {
                return (<sha2::Sha224 as sha2::Digest>::digest)((data)).to_vec();
            } else {
                if algorithm.clone() == "sha256".to_string() {
                    return (<sha2::Sha256 as sha2::Digest>::digest)((data)).to_vec();
                } else {
                    if algorithm.clone() == "sha384".to_string() {
                        return (<sha2::Sha384 as sha2::Digest>::digest)((data)).to_vec();
                    } else {
                        if algorithm.clone() == "sha512".to_string() {
                            return (<sha2::Sha512 as sha2::Digest>::digest)((data))
                                .to_vec();
                        } else {
                            if algorithm.clone() == "blake2b".to_string() {
                                return (<blake2::Blake2b512 as blake2::Digest>::digest)(
                                        (data),
                                    )
                                    .to_vec();
                            } else {
                                if algorithm.clone() == "blake2s".to_string() {
                                    return (<blake2::Blake2s256 as blake2::Digest>::digest)(
                                            (data),
                                        )
                                        .to_vec();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    return vec![];
}
fn _hash_hex(algorithm: &String, data: &Vec<u8>) -> String {
    return _hash_bytes(algorithm, data)
        .iter()
        .map(|__byte| format!("{:02x}", * __byte))
        .collect::<Vec<String>>()
        .join("");
}
fn new_bytes(name: &String, data: &Vec<u8>) -> Result<HashObject, ValueError> {
    if !(_is_supported_algorithm(name)) {
        return Err(
            ValueError::new(
                format!("{}{}", "unsupported hash algorithm: ".to_string(), name),
            ),
        );
    }
    return Ok(_build_hash(name, data));
}

// --- stdlib: sifr.base64 ---
fn b64encode_bytes(data: &Vec<u8>) -> Vec<u8> {
    return base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data)
        .into_bytes();
}
fn b64decode_bytes(data: &Vec<u8>) -> Result<Vec<u8>, ParseError> {
    return base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &data)
        .map_err(|e| ParseError {
            message: e.to_string(),
        });
}

#[derive(Debug, Clone)]
struct IOError {
    message: String,
    kind: String,
}

impl IOError {
    fn new(message: String) -> Self {
        return Self { message: message, kind: "Other".to_string() };
    }
}

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for IOError {
}

fn __io_err(e: std::io::Error) -> IOError {
    let msg = e.to_string();
    let kind = if e.kind() == std::io::ErrorKind::NotFound { "FileNotFound".to_string() } else { if e.kind() == std::io::ErrorKind::PermissionDenied { "PermissionDenied".to_string() } else { if e.kind() == std::io::ErrorKind::AlreadyExists { "FileExists".to_string() } else { "Other".to_string() } } };
    return IOError { message: msg, kind: kind };
}

#[derive(Debug, Clone)]
struct ParseError {
    message: String,
}

impl ParseError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for ParseError {
}

#[derive(Debug, Clone)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for ValueError {
}

#[derive(Debug, Clone)]
struct JSONDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl JSONDecodeError {
    fn new(message: String) -> Self {
        return Self { message: message, line: 0, column: 0 };
    }
}

impl std::fmt::Display for JSONDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for JSONDecodeError {
}

#[derive(Debug, Clone)]
struct TOMLDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl TOMLDecodeError {
    fn new(message: String) -> Self {
        return Self { message: message, line: 0, column: 0 };
    }
}

impl std::fmt::Display for TOMLDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for TOMLDecodeError {
}

#[derive(Debug, Clone)]
struct RegexError {
    message: String,
    detail: String,
}

impl RegexError {
    fn new(message: String) -> Self {
        return Self { message: message, detail: String::new() };
    }
}

impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for RegexError {
}

fn main() {
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let data: Vec<u8> = {
    let __s = "binary_hashing-bytes-demo".to_string();
    __s.as_bytes().to_vec()
};
    let mut h: HashObject = new_bytes(&"sha256".to_string(), &data)?;
    assert!((h.digest().len() as i64) == (32 as i64));
    assert!((h.hexdigest().chars().count() as i64) == (64 as i64));
    let enc: Vec<u8> = b64encode_bytes(&data);
    let dec: Vec<u8> = (b64decode_bytes(&enc)).map_err(|__e| ValueError::new(__e.to_string()))?;
    assert!(dec == data);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        assert!(format!("{}", format!("{}{}", "unexpected value error: ".to_string(), e.message)) == "rng_binary_hashing_base64_bytes_demo: pass".to_string());
    }
    assert!(format!("{}", "rng_binary_hashing_base64_bytes_demo: pass".to_string()) == "rng_binary_hashing_base64_bytes_demo: pass".to_string());
}
