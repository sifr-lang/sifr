#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FileResource {
    path: String,
}

impl FileResource {
    fn new(path: String) -> Self {
        return Self { path: path };
    }
    fn __enter__(&self) -> FileResource {
        return self.clone();
    }
    fn __exit__(&self) {
        return;
    }
}

impl std::fmt::Display for FileResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "FileResource(path={})", self.path);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DBConnection {
    name: String,
}

impl DBConnection {
    fn new(name: String) -> Self {
        return Self { name: name };
    }
    fn __enter__(&self) -> DBConnection {
        return self.clone();
    }
    fn __exit__(&self) {
        return;
    }
}

impl std::fmt::Display for DBConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "DBConnection(name={})", self.name);
    }
}

struct Config {
    value: i64,
    callback: Box<dyn Fn(i64) -> i64>,
}

impl Config {
    fn new(value: i64, callback: impl Fn(i64) -> i64 + 'static) -> Self {
        return Self { value: value, callback: Box::new(callback) };
    }
}

fn double(x: i64) -> i64 {
    return x * (2 as i64);
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
        output.push(format!("{}{}", "Reading: ".to_string(), f.path));
    }
    output.push("Closing: data.csv".to_string());
    output.push("42".to_string());
    return output;
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
        events.push(format!("{}{}", "Using: ".to_string(), f.path));
    }
    events.push("Closing: config.json".to_string());
    events.push("=== Context Manager: Early Return ===".to_string());
    let result_events: Vec<String> = demo_early_return();
    for item in result_events.iter().cloned() {
        events.push(item);
    }
    events.push("=== Context Manager: Break in Loop ===".to_string());
    let mut i: i64 = 0 as i64;
    while i < (3 as i64) {
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
            if i == (1 as i64) {
                should_break = true;
            } else {
                events.push(format!("{}{}", "Query on: ".to_string(), conn.name));
            }
        }
        events.push("Disconnecting: db".to_string());
        if should_break {
            break;
        }
        i = i + (1 as i64);
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
        events.push(format!("{}{}", format!("{}{}", format!("{}{}", "Processing with: ".to_string(), fin.path), " and ".to_string()), db.name));
    }
    events.push("Disconnecting: postgres".to_string());
    events.push("Closing: input.txt".to_string());
    events.push("=== Callable Struct Field ===".to_string());
    let c: Config = Config::new(21 as i64, double);
    events.push(format!("{}", c.value));
    events.push("=== Compiler Hardening Demo Complete ===".to_string());
    assert!(events == vec!["=== Context Manager: Normal Exit ===".to_string(), "Opening: config.json".to_string(), "Using: config.json".to_string(), "Closing: config.json".to_string(), "=== Context Manager: Early Return ===".to_string(), "Opening: data.csv".to_string(), "Reading: data.csv".to_string(), "Closing: data.csv".to_string(), "42".to_string(), "=== Context Manager: Break in Loop ===".to_string(), "Connecting: db".to_string(), "Query on: db".to_string(), "Disconnecting: db".to_string(), "Connecting: db".to_string(), "Disconnecting: db".to_string(), "=== Multiple Context Managers ===".to_string(), "Opening: input.txt".to_string(), "Connecting: postgres".to_string(), "Processing with: input.txt and postgres".to_string(), "Disconnecting: postgres".to_string(), "Closing: input.txt".to_string(), "=== Callable Struct Field ===".to_string(), "21".to_string(), "=== Compiler Hardening Demo Complete ===".to_string()]);
    println!("Compiler hardening demo trace:");
    for entry in events.iter().cloned() {
        println!("{}", entry);
    }
}
