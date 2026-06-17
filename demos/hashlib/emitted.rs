// --- stdlib: sifr.test ---
fn assert_eq<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    actual: &T,
    expected: &T,
) {
    assert!(* actual == * expected);
}
fn assert_vector_eq(actual: &Vec<String>, expected: &Vec<String>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0 as i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize].clone()) == expected.get(i as usize).cloned());
        i = i + (1 as i64);
    }
}
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0 as i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize]) == expected.get(i as usize).copied());
        i = i + (1 as i64);
    }
}

// --- stdlib: sifr.hashlib ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct HashlibError {
    message: String,
}
impl HashlibError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}
impl std::fmt::Display for HashlibError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.message);
    }
}
impl std::error::Error for HashlibError {}
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
fn copy_hash(h: &HashObject) -> HashObject {
    return _build_hash(&h._algorithm, &h._data);
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
fn new(name: &String, data: &String) -> Result<HashObject, ValueError> {
    return new_bytes(
        name,
        &({
            let __s = data;
            __s.as_bytes().to_vec()
        }),
    );
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
fn algorithms_guaranteed() -> Vec<String> {
    return vec![
        "md5".to_string(), "sha1".to_string(), "sha224".to_string(), "sha256"
        .to_string(), "sha384".to_string(), "sha512".to_string(), "blake2b".to_string(),
        "blake2s".to_string()
    ];
}
fn file_digest(path: &String, name: &String) -> Result<String, HashlibError> {
    let __sifr_try_res: Result<Result<String, HashlibError>, IOError> = (|| {
        let data: String = std::fs::read_to_string(&path).map_err(__io_err)?;
        let mut h: HashObject = (new(name, &data))
            .map_err(|__e| IOError::new(__e.to_string()))?;
        return Ok(Ok(h.hexdigest()));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(HashlibError::new(e.message));
        }
    }
}
fn md5_obj(data: &String) -> HashObject {
    return _build_hash(
        &"md5".to_string(),
        &({
            let __s = data;
            __s.as_bytes().to_vec()
        }),
    );
}
fn sha256_obj(data: &String) -> HashObject {
    return _build_hash(
        &"sha256".to_string(),
        &({
            let __s = data;
            __s.as_bytes().to_vec()
        }),
    );
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
struct Error {
    message: String,
}

impl Error {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for Error {
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

fn contains(values: &Vec<String>, needle: &String) -> bool {
    for v in values.iter().cloned() {
        if v == *needle {
            return true;
        }
    }
    return false;
}

fn collect_positive_actual(tmp_path: &String) -> Vec<String> {
    let mut actual: Vec<String> = vec![];
    let mut h: HashObject = sha256_obj(&"".to_string());
    h.update(&"a".to_string());
    h.update(&"bc".to_string());
    actual.push(format!("{}", (h.hexdigest()).as_str() == ((<sha2::Sha256 as sha2::Digest>::digest)(("abc".to_string()).as_bytes()).iter().map(|__byte| format!("{:02x}", *__byte)).collect::<Vec<String>>().join("".to_string().as_str())).as_str()));
    actual.push(format!("{}", ((h.digest().len() as i64) == (32 as i64)) && (h.digest_bytes() == h.digest())));
    let mut c: HashObject = copy_hash(&h);
    c.update(&"x".to_string());
    actual.push(format!("{}", (c.hexdigest()).as_str() == ((<sha2::Sha256 as sha2::Digest>::digest)(("abcx".to_string()).as_bytes()).iter().map(|__byte| format!("{:02x}", *__byte)).collect::<Vec<String>>().join("".to_string().as_str())).as_str()));
    let mut m: HashObject = md5_obj(&"hello".to_string());
    actual.push(format!("{}", (m.hexdigest()).as_str() == (md5::compute(("hello".to_string()).as_bytes()).0.iter().map(|__byte| format!("{:02x}", *__byte)).collect::<Vec<String>>().join("".to_string().as_str())).as_str()));
    actual.push(format!("{}", contains(&algorithms_guaranteed(), &"sha256".to_string())));
    actual.push(m.hexdigest());
    let __sifr_try_res: Result<(), HashlibError> = (|| {
    let out: String = file_digest(tmp_path, &"sha256".to_string())?;
    actual.push(out);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        actual.push("ERR".to_string());
    }
    return actual;
}

fn collect_negative_actual_ok() -> Vec<bool> {
    let mut actual_ok: Vec<bool> = vec![];
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let bad: HashObject = new(&"sha3_256".to_string(), &"".to_string())?;
    let _: String = format!("{}", bad.name);
    actual_ok.push(true);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        actual_ok.push(false);
    }
    return actual_ok;
}

fn main() {
    let expected: Vec<String> = vec!["true".to_string(), "true".to_string(), "true".to_string(), "true".to_string(), "true".to_string(), "5d41402abc4b2a76b9719d911017c592".to_string(), "8e6537b695ff181bc341e32d8b8970485ac3513408e5eb1e8ba9fc5af1cd3f57".to_string()];
    let tmp_path: String = "tmp_hashlib_hashlib_demo.txt".to_string();
    let _: Result<(), IOError> = std::fs::write(&tmp_path, "file-data".to_string().as_bytes()).map(|_| ()).map_err(__io_err);
    let actual: Vec<String> = collect_positive_actual(&tmp_path);
    assert_vector_eq(&actual, &expected);
    let expected_ok: Vec<bool> = vec![false];
    let actual_ok: Vec<bool> = collect_negative_actual_ok();
    assert_bool_vector_eq(&actual_ok, &expected_ok);
    println!("hashlib hashlib parity demo: pass");
}
