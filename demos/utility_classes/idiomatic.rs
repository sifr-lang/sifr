use std::collections::{BTreeMap, BTreeSet};
use std::net::Ipv4Addr;

use uuid::Uuid;

#[derive(Default)]
struct Namespace {
    values: BTreeMap<String, String>,
    flags: BTreeMap<String, bool>,
}

impl Namespace {
    fn get(&self, key: &str, default: &str) -> String {
        self.values
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    fn get_bool(&self, key: &str, default: bool) -> bool {
        self.flags.get(key).copied().unwrap_or(default)
    }
}

enum ArgumentKind {
    Flag,
    Option { default: String },
    Positional { default: String },
}

struct ArgumentSpec {
    name: String,
    dest: String,
    kind: ArgumentKind,
}

struct ArgumentParser {
    specs: Vec<ArgumentSpec>,
}

impl ArgumentParser {
    fn new() -> Self {
        Self { specs: Vec::new() }
    }

    fn add_flag(&mut self, name: &str, dest: &str) {
        self.specs.push(ArgumentSpec {
            name: name.to_string(),
            dest: dest.to_string(),
            kind: ArgumentKind::Flag,
        });
    }

    fn add_option(&mut self, name: &str, dest: &str, default: &str) {
        self.specs.push(ArgumentSpec {
            name: name.to_string(),
            dest: dest.to_string(),
            kind: ArgumentKind::Option {
                default: default.to_string(),
            },
        });
    }

    fn add_positional(&mut self, name: &str, default: &str) {
        self.specs.push(ArgumentSpec {
            name: name.to_string(),
            dest: name.to_string(),
            kind: ArgumentKind::Positional {
                default: default.to_string(),
            },
        });
    }

    fn parse_args(&self, args: &[&str]) -> Namespace {
        let mut parsed = Namespace::default();
        let mut positional_specs = Vec::new();

        for spec in &self.specs {
            match &spec.kind {
                ArgumentKind::Flag => {
                    parsed.flags.insert(spec.dest.clone(), false);
                }
                ArgumentKind::Option { default } | ArgumentKind::Positional { default } => {
                    parsed.values.insert(spec.dest.clone(), default.clone());
                    if matches!(spec.kind, ArgumentKind::Positional { .. }) {
                        positional_specs.push(spec.dest.clone());
                    }
                }
            }
        }

        let mut positional_index = 0usize;
        let mut i = 0usize;
        let mut literal_mode = false;

        while i < args.len() {
            let token = args[i];
            if !literal_mode && token == "--" {
                literal_mode = true;
                i += 1;
                continue;
            }

            if !literal_mode && token.starts_with("--") {
                if let Some((name, value)) = token.split_once('=') {
                    if let Some(spec) = self.specs.iter().find(|spec| spec.name == name) {
                        parsed.values.insert(spec.dest.clone(), value.to_string());
                    }
                    i += 1;
                    continue;
                }

                if let Some(spec) = self.specs.iter().find(|spec| spec.name == token) {
                    match &spec.kind {
                        ArgumentKind::Flag => {
                            parsed.flags.insert(spec.dest.clone(), true);
                        }
                        ArgumentKind::Option { .. } => {
                            if let Some(next) = args.get(i + 1) {
                                if !next.starts_with("--") {
                                    parsed.values.insert(spec.dest.clone(), (*next).to_string());
                                    i += 1;
                                }
                            }
                        }
                        ArgumentKind::Positional { .. } => {}
                    }
                }
                i += 1;
                continue;
            }

            if let Some(dest) = positional_specs.get(positional_index) {
                parsed.values.insert(dest.clone(), token.to_string());
                positional_index += 1;
            }
            i += 1;
        }

        parsed
    }
}

struct IPv4Address {
    addr: Ipv4Addr,
}

impl IPv4Address {
    fn to_str(&self) -> String {
        self.addr.to_string()
    }

    fn is_link_local(&self) -> bool {
        self.addr.is_link_local()
    }

    fn is_global(&self) -> bool {
        !(self.addr.is_private()
            || self.addr.is_loopback()
            || self.addr.is_link_local()
            || self.addr.is_broadcast()
            || self.addr.is_documentation()
            || self.addr.is_unspecified())
    }
}

fn ip_address(text: &str) -> Result<IPv4Address, String> {
    text.parse::<Ipv4Addr>()
        .map(|addr| IPv4Address { addr })
        .map_err(|error| error.to_string())
}

struct SimpleUuid {
    value: Uuid,
}

impl SimpleUuid {
    fn version(&self) -> usize {
        self.value.get_version_num() as usize
    }

    fn to_str(&self) -> String {
        self.value.hyphenated().to_string()
    }
}

fn uuid4_obj() -> SimpleUuid {
    SimpleUuid {
        value: Uuid::new_v4(),
    }
}

fn uuid_from_hex(text: &str) -> Result<SimpleUuid, String> {
    let normalized = text.trim_matches(|ch| ch == '{' || ch == '}');
    Uuid::parse_str(normalized)
        .map(|value| SimpleUuid { value })
        .map_err(|error| error.to_string())
}

struct TopologicalSorter {
    deps: BTreeMap<i64, BTreeSet<i64>>,
}

impl TopologicalSorter {
    fn new() -> Self {
        Self {
            deps: BTreeMap::new(),
        }
    }

    fn add_many(&mut self, node: i64, deps: &[i64]) {
        self.deps.entry(node).or_default();
        for dep in deps {
            self.deps.entry(*dep).or_default();
            self.deps.entry(node).or_default().insert(*dep);
        }
    }

    fn add(&mut self, node: i64, dep: i64) {
        self.add_many(node, &[dep]);
    }

    fn static_order(&self) -> Result<Vec<i64>, String> {
        let mut remaining = self.deps.clone();
        let mut ordered = Vec::new();

        while !remaining.is_empty() {
            let ready = remaining
                .iter()
                .filter(|(_, deps)| deps.is_empty())
                .map(|(node, _)| *node)
                .collect::<Vec<_>>();

            if ready.is_empty() {
                return Err("cycle detected in graph".to_string());
            }

            for node in ready {
                ordered.push(node);
                remaining.remove(&node);
                for deps in remaining.values_mut() {
                    deps.remove(&node);
                }
            }
        }

        Ok(ordered)
    }
}

fn main() {
    let mut parser = ArgumentParser::new();
    parser.add_flag("--strict", "strict");
    parser.add_option("--mode", "mode", "safe");
    parser.add_positional("entry", "demo.sifr");

    let parsed = parser.parse_args(&["--strict", "--mode", "parity", "main.sifr"]);
    println!("argparse.strict = {}", parsed.get_bool("strict", false));
    println!("argparse.mode = {}", parsed.get("mode", ""));
    println!("argparse.entry = {}", parsed.get("entry", ""));

    let parsed_inline = parser.parse_args(&["--mode=inline", "--", "--literal.sifr"]);
    println!("argparse.inline = {}", parsed_inline.get("mode", ""));
    println!("argparse.literal = {}", parsed_inline.get("entry", ""));

    let parsed_missing = parser.parse_args(&["--mode", "--strict", "fallback.sifr"]);
    println!("argparse.missing_mode = {}", parsed_missing.get("mode", ""));
    println!(
        "argparse.missing_strict = {}",
        parsed_missing.get_bool("strict", false)
    );

    match ip_address("8.8.8.8") {
        Ok(addr) => {
            println!(
                "ipaddress.value = {} global={}",
                addr.to_str(),
                addr.is_global()
            );
            let link_local = ip_address("169.254.10.20").expect("valid IPv4 literal");
            println!(
                "ipaddress.link_local = {} global={}",
                link_local.is_link_local(),
                link_local.is_global()
            );
            let multicast = ip_address("224.0.0.1").expect("valid IPv4 literal");
            println!("ipaddress.multicast_global = {}", multicast.is_global());
        }
        Err(error) => println!("ipaddress.error = {}", error),
    }

    let generated = uuid4_obj();
    println!(
        "uuid.version = {} text={}",
        generated.version(),
        generated.to_str()
    );
    match uuid_from_hex("{550E8400-E29B-41D4-A716-446655440000}") {
        Ok(parsed_curly) => println!("uuid.curly.parse = {}", parsed_curly.to_str()),
        Err(error) => println!("uuid.error = {}", error),
    }

    let mut sorter = TopologicalSorter::new();
    sorter.add_many(50, &[30, 40]);
    sorter.add(30, 10);
    sorter.add(40, 10);
    sorter.add_many(10, &[]);
    match sorter.static_order() {
        Ok(order) => println!("graphlib.order = {:?}", order),
        Err(error) => println!("graphlib.error = {}", error),
    }
}
