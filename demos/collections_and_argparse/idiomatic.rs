use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Counter {
    counts: HashMap<String, i64>,
}

impl Counter {
    fn new(iterable: &[&str]) -> Self {
        let mut counts = HashMap::new();
        for value in iterable {
            *counts.entry((*value).to_string()).or_insert(0) += 1;
        }
        Self { counts }
    }

    fn get(&self, key: &str) -> i64 {
        self.counts.get(key).copied().unwrap_or(0)
    }
}

#[derive(Debug, Clone)]
struct DefaultDict {
    default: i64,
    values: HashMap<String, i64>,
}

impl DefaultDict {
    fn new(default: i64) -> Self {
        Self {
            default,
            values: HashMap::new(),
        }
    }

    fn ensure(&mut self, key: &str) -> i64 {
        *self.values.entry(key.to_string()).or_insert(self.default)
    }

    fn set(&mut self, key: &str, value: i64) {
        self.values.insert(key.to_string(), value);
    }
}

#[derive(Debug, Clone)]
enum NamespaceValue {
    Bool(bool),
    Single(String),
    List(Vec<String>),
}

#[derive(Debug, Clone, Default)]
struct Namespace {
    values: HashMap<String, NamespaceValue>,
}

impl Namespace {
    fn set_bool(&mut self, key: &str, value: bool) {
        self.values
            .insert(key.to_string(), NamespaceValue::Bool(value));
    }

    fn set_single(&mut self, key: &str, value: String) {
        self.values
            .insert(key.to_string(), NamespaceValue::Single(value));
    }

    fn set_list(&mut self, key: &str, value: Vec<String>) {
        self.values
            .insert(key.to_string(), NamespaceValue::List(value));
    }

    fn get(&self, key: &str, default: &str) -> String {
        match self.values.get(key) {
            Some(NamespaceValue::Single(value)) => value.clone(),
            _ => default.to_string(),
        }
    }

    fn get_bool(&self, key: &str, default: bool) -> bool {
        match self.values.get(key) {
            Some(NamespaceValue::Bool(value)) => *value,
            _ => default,
        }
    }

    fn get_list(&self, key: &str) -> Vec<String> {
        match self.values.get(key) {
            Some(NamespaceValue::List(value)) => value.clone(),
            _ => Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct ArgumentSpec {
    flag: Option<String>,
    dest: String,
    action: String,
    nargs: Option<String>,
}

#[derive(Debug, Clone)]
struct ArgumentParser {
    name: String,
    subcommand_dest: Option<String>,
    options: Vec<ArgumentSpec>,
    positionals: Vec<ArgumentSpec>,
    subparsers: HashMap<String, ArgumentParser>,
}

impl ArgumentParser {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            subcommand_dest: None,
            options: Vec::new(),
            positionals: Vec::new(),
            subparsers: HashMap::new(),
        }
    }

    fn add_subparsers(&mut self, dest: &str) {
        self.subcommand_dest = Some(dest.to_string());
    }

    fn add_argument_typed(
        &mut self,
        flag: &str,
        dest: &str,
        action: &str,
        _default: Option<&str>,
        nargs: Option<&str>,
    ) {
        let spec = ArgumentSpec {
            flag: if flag.starts_with("--") {
                Some(flag.to_string())
            } else {
                None
            },
            dest: dest.to_string(),
            action: action.to_string(),
            nargs: nargs.map(ToString::to_string),
        };
        if spec.flag.is_some() {
            self.options.push(spec);
        } else {
            self.positionals.push(spec);
        }
    }

    fn add_parser(&mut self, name: &str, parser: ArgumentParser) {
        self.subparsers.insert(name.to_string(), parser);
    }

    fn parse_args(&self, args: &[&str]) -> Namespace {
        let _ = &self.name;
        let mut namespace = Namespace::default();
        if let Some((subcommand, rest)) = args.split_first() {
            if let Some(dest) = &self.subcommand_dest {
                namespace.set_single(dest, (*subcommand).to_string());
            }
            if let Some(parser) = self.subparsers.get(*subcommand) {
                parser.parse_into(rest, &mut namespace);
            }
        }
        namespace
    }

    fn parse_into(&self, args: &[&str], namespace: &mut Namespace) {
        let mut index = 0;
        while index < args.len() {
            let current = args[index];
            if let Some(spec) = self
                .options
                .iter()
                .find(|spec| spec.flag.as_deref() == Some(current))
            {
                if spec.action == "store_true" {
                    namespace.set_bool(&spec.dest, true);
                    index += 1;
                } else {
                    if index + 1 >= args.len() {
                        return;
                    }
                    namespace.set_single(&spec.dest, args[index + 1].to_string());
                    index += 2;
                }
            } else {
                break;
            }
        }

        for positional in &self.positionals {
            if positional.nargs.as_deref() == Some("+") {
                namespace.set_list(
                    &positional.dest,
                    args[index..]
                        .iter()
                        .map(|value| (*value).to_string())
                        .collect(),
                );
                break;
            }
        }
    }
}

fn main() {
    let counter = Counter::new(&["parse", "parse", "emit"]);
    assert_eq!(counter.get("parse"), 2);

    let mut attempts = DefaultDict::new(0);
    let current_attempts = attempts.ensure("collections_and_argparse");
    attempts.set("collections_and_argparse", current_attempts + 1);
    assert_eq!(attempts.ensure("collections_and_argparse"), 1);

    let mut parser = ArgumentParser::new("sifr");
    parser.add_subparsers("cmd");
    let mut run_parser = ArgumentParser::new("run");
    run_parser.add_argument_typed("--strict", "strict", "store_true", None, None);
    run_parser.add_argument_typed("--level", "level", "store", Some("0"), Some("1"));
    run_parser.add_argument_typed(
        "--custom-level",
        "custom_level",
        "store",
        Some("0"),
        Some("1"),
    );
    run_parser.add_argument_typed("targets", "targets", "store", Some(""), Some("+"));
    parser.add_parser("run", run_parser);

    let parsed = parser.parse_args(&[
        "run",
        "--strict",
        "--level",
        "2",
        "--custom-level",
        "3",
        "main.sifr",
    ]);
    assert_eq!(parsed.get("cmd", ""), "run");
    assert!(parsed.get_bool("strict", false));
    assert_eq!(parsed.get("level", ""), "2");
    assert_eq!(parsed.get("custom_level", ""), "3");
    assert_eq!(
        format!("{:?}", parsed.get_list("targets")),
        "[\"main.sifr\"]"
    );
}
