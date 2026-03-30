use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Default)]
struct EventLog(Rc<RefCell<Vec<String>>>);

impl EventLog {
    fn push(&self, event: impl Into<String>) {
        self.0.borrow_mut().push(event.into());
    }

    fn snapshot(&self) -> Vec<String> {
        self.0.borrow().clone()
    }
}

struct FileResource {
    path: &'static str,
    events: EventLog,
}

impl FileResource {
    fn open(events: &EventLog, path: &'static str) -> Self {
        events.push(format!("Opening: {path}"));
        Self {
            path,
            events: events.clone(),
        }
    }

    fn path(&self) -> &'static str {
        self.path
    }
}

impl Drop for FileResource {
    fn drop(&mut self) {
        self.events.push(format!("Closing: {}", self.path));
    }
}

struct DbConnection {
    name: &'static str,
    events: EventLog,
}

impl DbConnection {
    fn connect(events: &EventLog, name: &'static str) -> Self {
        events.push(format!("Connecting: {name}"));
        Self {
            name,
            events: events.clone(),
        }
    }

    fn name(&self) -> &'static str {
        self.name
    }
}

impl Drop for DbConnection {
    fn drop(&mut self) {
        self.events.push(format!("Disconnecting: {}", self.name));
    }
}

struct Config {
    value: i64,
    callback: Box<dyn Fn(i64) -> i64>,
}

fn double(x: i64) -> i64 {
    x * 2
}

fn demo_early_return() -> Vec<String> {
    let events = EventLog::default();
    {
        let file = FileResource::open(&events, "data.csv");
        events.push(format!("Reading: {}", file.path()));
    }
    events.push("42");
    events.snapshot()
}

fn main() {
    let events = EventLog::default();

    events.push("=== Context Manager: Normal Exit ===");
    {
        let file = FileResource::open(&events, "config.json");
        events.push(format!("Using: {}", file.path()));
    }

    events.push("=== Context Manager: Early Return ===");
    for entry in demo_early_return() {
        events.push(entry);
    }

    events.push("=== Context Manager: Break in Loop ===");
    let mut i = 0;
    while i < 3 {
        {
            let conn = DbConnection::connect(&events, "db");
            if i != 1 {
                events.push(format!("Query on: {}", conn.name()));
            }
        }
        if i == 1 {
            break;
        }
        i += 1;
    }

    events.push("=== Multiple Context Managers ===");
    {
        let file = FileResource::open(&events, "input.txt");
        let db = DbConnection::connect(&events, "postgres");
        events.push(format!(
            "Processing with: {} and {}",
            file.path(),
            db.name()
        ));
    }

    events.push("=== Callable Struct Field ===");
    let config = Config {
        value: 21,
        callback: Box::new(double),
    };
    assert_eq!((config.callback)(config.value), 42);
    events.push(config.value.to_string());
    events.push("=== Compiler Hardening Demo Complete ===");

    let trace = events.snapshot();
    assert_eq!(
        trace,
        vec![
            "=== Context Manager: Normal Exit ===",
            "Opening: config.json",
            "Using: config.json",
            "Closing: config.json",
            "=== Context Manager: Early Return ===",
            "Opening: data.csv",
            "Reading: data.csv",
            "Closing: data.csv",
            "42",
            "=== Context Manager: Break in Loop ===",
            "Connecting: db",
            "Query on: db",
            "Disconnecting: db",
            "Connecting: db",
            "Disconnecting: db",
            "=== Multiple Context Managers ===",
            "Opening: input.txt",
            "Connecting: postgres",
            "Processing with: input.txt and postgres",
            "Disconnecting: postgres",
            "Closing: input.txt",
            "=== Callable Struct Field ===",
            "21",
            "=== Compiler Hardening Demo Complete ===",
        ]
    );

    println!("Compiler hardening demo trace:");
    for entry in trace {
        println!("{entry}");
    }
}
