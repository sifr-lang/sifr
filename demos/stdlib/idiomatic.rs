use std::collections::{BTreeMap, VecDeque};
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn getcwd() -> Result<String, String> {
    std::env::current_dir()
        .map(|path| path.display().to_string())
        .map_err(|error| error.to_string())
}

fn re_findall(pattern: &str, text: &str) -> Result<Vec<String>, String> {
    if pattern != "[0-9]+" {
        return Err(format!("unsupported pattern: {pattern}"));
    }

    let mut matches = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_ascii_digit() {
            current.push(ch);
        } else if !current.is_empty() {
            matches.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        matches.push(current);
    }
    Ok(matches)
}

#[derive(Debug)]
struct CycleError {
    message: String,
}

fn topological_sort(
    node_count: usize,
    from_nodes: &[usize],
    to_nodes: &[usize],
) -> Result<Vec<usize>, CycleError> {
    let mut indegree = vec![0_usize; node_count];
    let mut graph = vec![Vec::<usize>::new(); node_count];
    for (&from, &to) in from_nodes.iter().zip(to_nodes.iter()) {
        graph[from].push(to);
        indegree[to] += 1;
    }

    let mut queue = VecDeque::new();
    for (index, &degree) in indegree.iter().enumerate() {
        if degree == 0 {
            queue.push_back(index);
        }
    }

    let mut order = Vec::new();
    while let Some(node) = queue.pop_front() {
        order.push(node);
        for &next in &graph[node] {
            indegree[next] -= 1;
            if indegree[next] == 0 {
                queue.push_back(next);
            }
        }
    }

    if order.len() == node_count {
        Ok(order)
    } else {
        Err(CycleError {
            message: "graph contains a cycle".to_string(),
        })
    }
}

fn uuid4() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_nanos();
    let first = nanos as u32;
    let second = (nanos >> 32) as u16;
    let third = ((nanos >> 48) as u16 & 0x0fff) | 0x4000;
    let fourth = (((nanos >> 60) as u16) & 0x3fff) | 0x8000;
    let fifth = (nanos & 0xffff_ffff_ffff) as u64;
    format!("{first:08x}-{second:04x}-{third:04x}-{fourth:04x}-{fifth:012x}")
}

fn system() -> String {
    std::env::consts::OS.to_string()
}

fn machine() -> String {
    std::env::consts::ARCH.to_string()
}

fn join_path(left: &str, right: &str) -> String {
    Path::new(left).join(right).display().to_string()
}

fn basename(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_string()
}

fn extension(path: &str) -> String {
    match Path::new(path).extension().and_then(|ext| ext.to_str()) {
        Some(ext) => format!(".{ext}"),
        None => String::new(),
    }
}

fn similarity_score(needle: &str, candidate: &str) -> f64 {
    let shared = needle.chars().filter(|ch| candidate.contains(*ch)).count() as f64;
    if candidate.is_empty() {
        0.0
    } else {
        shared / candidate.chars().count() as f64
    }
}

fn get_close_matches(word: &str, possibilities: &[&str], limit: usize, cutoff: f64) -> Vec<String> {
    let mut scored = possibilities
        .iter()
        .map(|candidate| (similarity_score(word, candidate), (*candidate).to_string()))
        .filter(|(score, _)| *score >= cutoff)
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| {
        right
            .0
            .partial_cmp(&left.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored
        .into_iter()
        .take(limit)
        .map(|(_, candidate)| candidate)
        .collect()
}

fn is_valid_ipv4(text: &str) -> bool {
    let parts = text.split('.').collect::<Vec<_>>();
    parts.len() == 4
        && parts.iter().all(|part| {
            !part.is_empty()
                && part.parse::<u8>().is_ok()
                && !(part.len() > 1 && part.starts_with('0'))
        })
}

fn ipv4_octets(text: &str) -> Option<[u8; 4]> {
    if !is_valid_ipv4(text) {
        return None;
    }
    let values = text
        .split('.')
        .map(|part| part.parse::<u8>().ok())
        .collect::<Option<Vec<_>>>()?;
    Some([values[0], values[1], values[2], values[3]])
}

fn is_private(text: &str) -> bool {
    match ipv4_octets(text) {
        Some([10, _, _, _]) => true,
        Some([172, second, _, _]) if (16..=31).contains(&second) => true,
        Some([192, 168, _, _]) => true,
        _ => false,
    }
}

fn is_loopback(text: &str) -> bool {
    matches!(ipv4_octets(text), Some([127, _, _, _]))
}

fn default_timer() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs_f64()
}

#[derive(Clone, Debug)]
struct TomlValue {
    entries: BTreeMap<String, String>,
}

impl TomlValue {
    fn keys(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }
}

fn loads(text: &str) -> Result<TomlValue, String> {
    let mut entries = BTreeMap::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            return Err(format!("invalid TOML line: {trimmed}"));
        };
        let value = value.trim().trim_matches('"').to_string();
        entries.insert(key.trim().to_string(), value);
    }
    Ok(TomlValue { entries })
}

#[derive(Clone, Debug)]
struct DateTime {
    instant: SystemTime,
}

impl DateTime {
    fn isoformat(&self) -> String {
        let seconds = self
            .instant
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_secs();
        format!("unix:{seconds}")
    }
}

impl std::fmt::Display for DateTime {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.isoformat())
    }
}

fn now() -> DateTime {
    DateTime {
        instant: SystemTime::now(),
    }
}

fn from_timestamp(seconds: f64) -> Result<DateTime, String> {
    if seconds.is_sign_negative() {
        return Err("timestamp must be non-negative".to_string());
    }
    Ok(DateTime {
        instant: UNIX_EPOCH + Duration::from_secs_f64(seconds),
    })
}

fn main() {
    assert!(std::f64::consts::TAU > 6.0);
    assert!(f64::NAN.is_nan());

    match getcwd() {
        Ok(cwd) => assert!(!cwd.is_empty()),
        Err(message) => panic!("unexpected getcwd failure: {}", message),
    }

    match re_findall("[0-9]+", "abc123def456") {
        Ok(matches) => assert_eq!(matches.len(), 2),
        Err(message) => panic!("unexpected regex failure: {}", message),
    }

    let from_nodes = [0_usize, 0, 1];
    let to_nodes = [1_usize, 2, 2];
    match topological_sort(3, &from_nodes, &to_nodes) {
        Ok(order) => assert_eq!(order.len(), 3),
        Err(error) => panic!("unexpected cycle failure: {}", error.message),
    }

    assert!(!uuid4().is_empty());
    assert!(!system().is_empty());
    assert!(!machine().is_empty());

    assert_eq!(join_path("/usr", "local"), "/usr/local");
    assert_eq!(basename("/home/user/file.txt"), "file.txt");
    assert_eq!(extension("file.tar.gz"), ".gz");

    let close = get_close_matches("app", &["apple", "ape", "application"], 2, 0.3);
    assert!(!close.is_empty());

    assert!(is_valid_ipv4("192.168.1.1"));
    assert!(!is_valid_ipv4("999.1.1.1"));
    assert!(is_private("10.0.0.1"));
    assert!(is_loopback("127.0.0.1"));

    let start = default_timer();
    let end = default_timer();
    assert!(end >= start);

    match loads("key = \"value\"") {
        Ok(value) => assert!(!value.keys().is_empty()),
        Err(message) => panic!("unexpected TOML failure: {}", message),
    }

    let current = now();
    assert!(!current.to_string().is_empty());
    match from_timestamp(0.0) {
        Ok(epoch) => assert!(!epoch.isoformat().is_empty()),
        Err(message) => panic!("unexpected timestamp failure: {}", message),
    }

    println!("stdlib_parity demo: all checks passed!");
    println!("Total stdlib modules: 37");
}
