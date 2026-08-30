// src/main.rs
use ::sifr_runtime::SifrInt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FileResource {
    path: String,
}

impl FileResource {
    fn new(path: String) -> Self {
        let __sifr_field_init_0: String = path;
        Self { path: __sifr_field_init_0 }
    }
}

impl FileResource {
    fn __enter__(&self) -> FileResource {
        self.clone()
    }
}

impl FileResource {
    fn __exit__(&self) {
    }
}

impl ::std::fmt::Display for FileResource {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "FileResource(path={})", self.path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DBConnection {
    name: String,
}

impl DBConnection {
    fn new(name: String) -> Self {
        let __sifr_field_init_0: String = name;
        Self { name: __sifr_field_init_0 }
    }
}

impl DBConnection {
    fn __enter__(&self) -> DBConnection {
        self.clone()
    }
}

impl DBConnection {
    fn __exit__(&self) {
    }
}

impl ::std::fmt::Display for DBConnection {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "DBConnection(name={})", self.name)
    }
}

struct Config {
    value: SifrInt,
    callback: Box<dyn Fn(SifrInt) -> SifrInt>,
}

impl Config {
    fn new(value: SifrInt, callback: impl Fn(SifrInt) -> SifrInt + 'static) -> Self {
        let __sifr_field_init_0: SifrInt = value.clone();
        let __sifr_field_init_1: Box<dyn Fn(SifrInt) -> SifrInt> = Box::new(callback);
        Self { value: __sifr_field_init_0, callback: __sifr_field_init_1 }
    }
}

impl Config {
}

fn double(x: SifrInt) -> SifrInt {
    &x * &SifrInt::from_i64(2)
}

fn demo_early_return() -> Vec<String> {
    let mut output: Vec<String> = vec![];
    output.push("Opening: data.csv".to_string());
    {
        let mut __ctx_0 = FileResource::new("data.csv".to_string());
        struct __WithGuard0 { ctx: FileResource }
        impl Drop for __WithGuard0 {
            fn drop(&mut self) { self.ctx.__exit__(); }
        }
        let mut __guard_0 = __WithGuard0 { ctx: __ctx_0 };
        let f = __guard_0.ctx.__enter__();
        output.push(format!("{}{}", "Reading: ", f.path.clone()));
    }
    output.push("Closing: data.csv".to_string());
    output.push("42".to_string());
    output
}

fn main() {
    let mut events: Vec<String> = vec![];
    events.push("=== Context Manager: Normal Exit ===".to_string());
    events.push("Opening: config.json".to_string());
    {
        let mut __ctx_0 = FileResource::new("config.json".to_string());
        struct __WithGuard0 { ctx: FileResource }
        impl Drop for __WithGuard0 {
            fn drop(&mut self) { self.ctx.__exit__(); }
        }
        let mut __guard_0 = __WithGuard0 { ctx: __ctx_0 };
        let f = __guard_0.ctx.__enter__();
        events.push(format!("{}{}", "Using: ", f.path.clone()));
    }
    events.push("Closing: config.json".to_string());
    events.push("=== Context Manager: Early Return ===".to_string());
    let result_events: Vec<String> = demo_early_return();
    for item in result_events.iter().cloned() {
        events.push(item.clone());
    }
    events.push("=== Context Manager: Break in Loop ===".to_string());
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from_i64(3)) {
        events.push("Connecting: db".to_string());
        let mut should_break: bool = false;
        {
            let mut __ctx_0 = DBConnection::new("db".to_string());
            struct __WithGuard0 { ctx: DBConnection }
            impl Drop for __WithGuard0 {
                fn drop(&mut self) { self.ctx.__exit__(); }
            }
            let mut __guard_0 = __WithGuard0 { ctx: __ctx_0 };
            let conn = __guard_0.ctx.__enter__();
            if (&i == &SifrInt::from_i64(1)) {
                should_break = true;
            } else {
                events.push(format!("{}{}", "Query on: ", conn.name.clone()));
            }
        }
        events.push("Disconnecting: db".to_string());
        if should_break {
            break;
        }
        i = &i + &SifrInt::from_i64(1);
    }
    events.push("=== Multiple Context Managers ===".to_string());
    events.push("Opening: input.txt".to_string());
    events.push("Connecting: postgres".to_string());
    {
        let mut __ctx_0 = FileResource::new("input.txt".to_string());
        struct __WithGuard0 { ctx: FileResource }
        impl Drop for __WithGuard0 {
            fn drop(&mut self) { self.ctx.__exit__(); }
        }
        let mut __guard_0 = __WithGuard0 { ctx: __ctx_0 };
        let fin = __guard_0.ctx.__enter__();
        let mut __ctx_1 = DBConnection::new("postgres".to_string());
        struct __WithGuard1 { ctx: DBConnection }
        impl Drop for __WithGuard1 {
            fn drop(&mut self) { self.ctx.__exit__(); }
        }
        let mut __guard_1 = __WithGuard1 { ctx: __ctx_1 };
        let db = __guard_1.ctx.__enter__();
        events.push(format!("{}{}", format!("{}{}", format!("{}{}", "Processing with: ", fin.path.clone()), " and "), db.name.clone()));
    }
    events.push("Disconnecting: postgres".to_string());
    events.push("Closing: input.txt".to_string());
    events.push("=== Callable Struct Field ===".to_string());
    let c: Config = Config::new(SifrInt::from_i64(21), double);
    events.push(format!("{}", c.value.clone()));
    events.push("=== Compiler Hardening Demo Complete ===".to_string());
    assert!((events == vec!["=== Context Manager: Normal Exit ===".to_string(), "Opening: config.json".to_string(), "Using: config.json".to_string(), "Closing: config.json".to_string(), "=== Context Manager: Early Return ===".to_string(), "Opening: data.csv".to_string(), "Reading: data.csv".to_string(), "Closing: data.csv".to_string(), "42".to_string(), "=== Context Manager: Break in Loop ===".to_string(), "Connecting: db".to_string(), "Query on: db".to_string(), "Disconnecting: db".to_string(), "Connecting: db".to_string(), "Disconnecting: db".to_string(), "=== Multiple Context Managers ===".to_string(), "Opening: input.txt".to_string(), "Connecting: postgres".to_string(), "Processing with: input.txt and postgres".to_string(), "Disconnecting: postgres".to_string(), "Closing: input.txt".to_string(), "=== Callable Struct Field ===".to_string(), "21".to_string(), "=== Compiler Hardening Demo Complete ===".to_string()]));
    println!("Compiler hardening demo trace:");
    for entry in events.iter().cloned() {
        println!("{}", entry);
    }
}
