// src/main.rs
use ::sifr_runtime::SifrInt;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FileResource {
    path: String,
}
impl FileResource {
    const fn new(path: String) -> Self {
        let sifr_generated_field_value_03c52d0debd70676_70617468: String = path;
        Self {
            path: sifr_generated_field_value_03c52d0debd70676_70617468,
        }
    }
}
impl FileResource {
    fn sifr_generated_enter__(&self) -> FileResource {
        self.clone()
    }
}
impl FileResource {
    const fn sifr_generated_exit__(&self) {}
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
    const fn new(name: String) -> Self {
        let sifr_generated_field_value_c4bcadba8e631b86_6e616d65: String = name;
        Self {
            name: sifr_generated_field_value_c4bcadba8e631b86_6e616d65,
        }
    }
}
impl DBConnection {
    fn sifr_generated_enter__(&self) -> DBConnection {
        self.clone()
    }
}
impl DBConnection {
    const fn sifr_generated_exit__(&self) {}
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
        let sifr_generated_field_value_7ce4fd9430e80cea_76616c7565: SifrInt = value.clone();
        let sifr_generated_field_value_31d52eaacb529206_63616c6c6261636b: Box<
            dyn Fn(SifrInt) -> SifrInt,
        > = Box::new(callback);
        Self {
            value: sifr_generated_field_value_7ce4fd9430e80cea_76616c7565,
            callback: sifr_generated_field_value_31d52eaacb529206_63616c6c6261636b,
        }
    }
}
fn double(x: SifrInt) -> SifrInt {
    &x * &SifrInt::from_i64(2)
}
fn demo_early_return() -> Vec<String> {
    let mut output: Vec<String> = vec!["Opening: data.csv".to_string()];
    {
        struct SifrGeneratedWithGuard0 {
            ctx: FileResource,
        }
        impl Drop for SifrGeneratedWithGuard0 {
            fn drop(&mut self) {
                self.ctx.sifr_generated_exit__();
            }
        }
        let sifr_generated_ctx_0 = FileResource::new("data.csv".to_string());
        let sifr_generated_guard_0 = SifrGeneratedWithGuard0 {
            ctx: sifr_generated_ctx_0,
        };
        let f = sifr_generated_guard_0.ctx.sifr_generated_enter__();
        output.push(format!("Reading: {}", f.path.clone()));
    }
    output.push("Closing: data.csv".to_string());
    output.push("42".to_string());
    output
}
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    let mut events: Vec<String> = vec![
        "=== Context Manager: Normal Exit ===".to_string(),
        "Opening: config.json".to_string(),
    ];
    {
        struct SifrGeneratedWithGuard0 {
            ctx: FileResource,
        }
        impl Drop for SifrGeneratedWithGuard0 {
            fn drop(&mut self) {
                self.ctx.sifr_generated_exit__();
            }
        }
        let sifr_generated_ctx_0 = FileResource::new("config.json".to_string());
        let sifr_generated_guard_0 = SifrGeneratedWithGuard0 {
            ctx: sifr_generated_ctx_0,
        };
        let f = sifr_generated_guard_0.ctx.sifr_generated_enter__();
        events.push(format!("Using: {}", f.path.clone()));
    }
    events.push("Closing: config.json".to_string());
    events.push("=== Context Manager: Early Return ===".to_string());
    let result_events: Vec<String> = demo_early_return();
    for item in result_events.iter().cloned() {
        events.push(item.to_owned());
    }
    events.push("=== Context Manager: Break in Loop ===".to_string());
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from_i64(3) {
        events.push("Connecting: db".to_string());
        let mut should_break: bool = false;
        {
            struct SifrGeneratedWithGuard0 {
                ctx: DBConnection,
            }
            impl Drop for SifrGeneratedWithGuard0 {
                fn drop(&mut self) {
                    self.ctx.sifr_generated_exit__();
                }
            }
            let sifr_generated_ctx_0 = DBConnection::new("db".to_string());
            let sifr_generated_guard_0 = SifrGeneratedWithGuard0 {
                ctx: sifr_generated_ctx_0,
            };
            let conn = sifr_generated_guard_0.ctx.sifr_generated_enter__();
            if &i == &SifrInt::from_i64(1) {
                should_break = true;
            } else {
                events.push(format!("Query on: {}", conn.name.clone()));
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
        struct SifrGeneratedWithGuard0 {
            ctx: FileResource,
        }
        impl Drop for SifrGeneratedWithGuard0 {
            fn drop(&mut self) {
                self.ctx.sifr_generated_exit__();
            }
        }
        struct SifrGeneratedWithGuard1 {
            ctx: DBConnection,
        }
        impl Drop for SifrGeneratedWithGuard1 {
            fn drop(&mut self) {
                self.ctx.sifr_generated_exit__();
            }
        }
        let sifr_generated_ctx_0 = FileResource::new("input.txt".to_string());
        let sifr_generated_guard_0 = SifrGeneratedWithGuard0 {
            ctx: sifr_generated_ctx_0,
        };
        let fin = sifr_generated_guard_0.ctx.sifr_generated_enter__();
        let sifr_generated_ctx_1 = DBConnection::new("postgres".to_string());
        let sifr_generated_guard_1 = SifrGeneratedWithGuard1 {
            ctx: sifr_generated_ctx_1,
        };
        let db = sifr_generated_guard_1.ctx.sifr_generated_enter__();
        events.push(format!(
            "Processing with: {} and {}",
            fin.path.clone(),
            db.name.clone()
        ));
    }
    events.push("Disconnecting: postgres".to_string());
    events.push("Closing: input.txt".to_string());
    events.push("=== Callable Struct Field ===".to_string());
    let c: Config = Config::new(SifrInt::from_i64(21), double);
    events.push(c.value.clone().to_string());
    events.push("=== Compiler Hardening Demo Complete ===".to_string());
    assert_eq!(
        events,
        vec![
            "=== Context Manager: Normal Exit ===".to_string(),
            "Opening: config.json".to_string(),
            "Using: config.json".to_string(),
            "Closing: config.json".to_string(),
            "=== Context Manager: Early Return ===".to_string(),
            "Opening: data.csv".to_string(),
            "Reading: data.csv".to_string(),
            "Closing: data.csv".to_string(),
            "42".to_string(),
            "=== Context Manager: Break in Loop ===".to_string(),
            "Connecting: db".to_string(),
            "Query on: db".to_string(),
            "Disconnecting: db".to_string(),
            "Connecting: db".to_string(),
            "Disconnecting: db".to_string(),
            "=== Multiple Context Managers ===".to_string(),
            "Opening: input.txt".to_string(),
            "Connecting: postgres".to_string(),
            "Processing with: input.txt and postgres".to_string(),
            "Disconnecting: postgres".to_string(),
            "Closing: input.txt".to_string(),
            "=== Callable Struct Field ===".to_string(),
            "21".to_string(),
            "=== Compiler Hardening Demo Complete ===".to_string()
        ]
    );
    println!("Compiler hardening demo trace:");
    for entry in events.iter().cloned() {
        println!("{entry}");
    }
}
