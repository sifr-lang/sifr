use std::collections::VecDeque;
use std::net::Ipv4Addr;

#[derive(Debug)]
struct ValueError {
    message: String,
}

#[derive(Debug)]
struct CycleError {
    message: String,
}

fn randint(min: i64, max: i64) -> Result<i64, ValueError> {
    if min > max {
        Err(ValueError {
            message: "randint: min must be <= max".to_string(),
        })
    } else {
        Ok(min)
    }
}

fn randbelow(n: i64) -> Result<i64, ValueError> {
    if n <= 0 {
        Err(ValueError {
            message: "randbelow: n must be > 0".to_string(),
        })
    } else {
        Ok(0)
    }
}

fn wrap(text: &str, width: usize) -> Result<Vec<String>, ValueError> {
    if width == 0 {
        return Err(ValueError {
            message: "wrap: width must be > 0".to_string(),
        });
    }

    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split(' ') {
        let candidate_len = if current.is_empty() {
            word.len()
        } else {
            current.len() + 1 + word.len()
        };
        if !current.is_empty() && candidate_len > width {
            lines.push(current);
            current = word.to_string();
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    Ok(lines)
}

fn batched<T: Clone>(items: &[T], size: usize) -> Result<Vec<Vec<T>>, ValueError> {
    if size == 0 {
        return Err(ValueError {
            message: "batched: n must be > 0".to_string(),
        });
    }

    Ok(items.chunks(size).map(|chunk| chunk.to_vec()).collect())
}

fn topological_sort(
    node_count: usize,
    from: &[usize],
    to: &[usize],
) -> Result<Vec<usize>, CycleError> {
    let mut edges = vec![Vec::new(); node_count];
    let mut indegree = vec![0_usize; node_count];

    for (&src, &dst) in from.iter().zip(to.iter()) {
        edges[src].push(dst);
        indegree[dst] += 1;
    }

    let mut ready: VecDeque<usize> = indegree
        .iter()
        .enumerate()
        .filter_map(|(node, degree)| (*degree == 0).then_some(node))
        .collect();
    let mut order = Vec::new();

    while let Some(node) = ready.pop_front() {
        order.push(node);
        for &next in &edges[node] {
            indegree[next] -= 1;
            if indegree[next] == 0 {
                ready.push_back(next);
            }
        }
    }

    if order.len() == node_count {
        Ok(order)
    } else {
        Err(CycleError {
            message: "cycle detected in graph".to_string(),
        })
    }
}

fn uuid_from_hex(value: &str) -> Result<(), ValueError> {
    if !value.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(ValueError {
            message: "invalid UUID hex string".to_string(),
        });
    }
    if value.len() != 32 {
        return Err(ValueError {
            message: "UUID hex string must be 32 hex characters".to_string(),
        });
    }
    Ok(())
}

fn ip_to_int(value: &str) -> Result<u32, ValueError> {
    value
        .parse::<Ipv4Addr>()
        .map(u32::from)
        .map_err(|_| ValueError {
            message: "invalid IPv4 address".to_string(),
        })
}

fn from_timestamp(timestamp: f64) -> Result<(), ValueError> {
    if !timestamp.is_finite() || !(-1.0e12..=1.0e12).contains(&timestamp) {
        Err(ValueError {
            message: "invalid timestamp".to_string(),
        })
    } else {
        Ok(())
    }
}

fn set_at<T>(items: &mut [T], index: usize, value: T) {
    if let Some(slot) = items.get_mut(index) {
        *slot = value;
    }
}

fn main() {
    println!("=== 1. random.randint: Validates a <= b ===");
    match randint(1, 10) {
        Ok(_) => println!("randint(1, 10) = ok"),
        Err(error) => println!("error: {}", error.message),
    }
    match randint(5, 3) {
        Ok(_) => println!("should not reach here"),
        Err(error) => println!("randint(5, 3) -> ValueError: {}", error.message),
    }

    println!("=== 2. secrets.randbelow: Validates n > 0 ===");
    match randbelow(100) {
        Ok(_) => println!("randbelow(100) = ok"),
        Err(error) => println!("error: {}", error.message),
    }
    match randbelow(0) {
        Ok(_) => println!("should not reach here"),
        Err(error) => println!("randbelow(0) -> ValueError: {}", error.message),
    }

    println!("=== 3. textwrap.wrap: Validates width > 0 ===");
    match wrap("hello world", 5) {
        Ok(lines) => println!("wrap(hello world, 5) = ok ({} lines)", lines.len()),
        Err(error) => println!("error: {}", error.message),
    }
    match wrap("hello", 0) {
        Ok(_) => println!("should not reach here"),
        Err(error) => println!("wrap(hello, 0) -> ValueError: {}", error.message),
    }

    println!("=== 4. itertools.batched: Validates n > 0 ===");
    let data = vec![1_i64, 2, 3, 4, 5];
    match batched(&data, 2) {
        Ok(groups) => println!("batched([1,2,3,4,5], 2) = ok ({} batches)", groups.len()),
        Err(error) => println!("error: {}", error.message),
    }
    match batched(&data, 0) {
        Ok(_) => println!("should not reach here"),
        Err(error) => println!("batched(data, 0) -> ValueError: {}", error.message),
    }

    println!("=== 5. graphlib.topological_sort: Cycle Detection ===");
    match topological_sort(3, &[0, 0], &[1, 2]) {
        Ok(order) => println!("acyclic graph: {:?}", order),
        Err(error) => println!("error: {}", error.message),
    }
    match topological_sort(2, &[0, 1], &[1, 0]) {
        Ok(_) => println!("should not reach here"),
        Err(error) => println!("cyclic graph -> CycleError: {}", error.message),
    }

    println!("=== 6. uuid.uuid_from_hex: Validates hex format ===");
    match uuid_from_hex("550e8400e29b41d4a716446655440000") {
        Ok(()) => println!("valid UUID hex: ok"),
        Err(error) => println!("error: {}", error.message),
    }
    match uuid_from_hex("xyz-invalid!") {
        Ok(()) => println!("should not reach here"),
        Err(error) => println!("invalid chars -> ValueError: {}", error.message),
    }
    match uuid_from_hex("abcd1234") {
        Ok(()) => println!("should not reach here"),
        Err(error) => println!("wrong length -> ValueError: {}", error.message),
    }

    println!("=== 7. ipaddress.ip_to_int: Validates IPv4 format ===");
    match ip_to_int("192.168.1.1") {
        Ok(_) => println!("ip_to_int(192.168.1.1) = ok"),
        Err(error) => println!("error: {}", error.message),
    }
    match ip_to_int("bad") {
        Ok(_) => println!("should not reach here"),
        Err(error) => println!("ip_to_int(bad) -> ValueError: {}", error.message),
    }

    println!("=== 8. datetime.from_timestamp: Validates timestamp ===");
    match from_timestamp(0.0) {
        Ok(()) => println!("from_timestamp(0.0) = ok"),
        Err(error) => println!("error: {}", error.message),
    }
    match from_timestamp(-99999999999999.0) {
        Ok(()) => println!("should not reach here"),
        Err(error) => println!("from_timestamp(invalid) -> ValueError: {}", error.message),
    }

    println!("=== 9. SubscriptAssign: Bounds-checked (safe no-op) ===");
    let mut nums = vec![10_i64, 20, 30];
    println!("before: {:?}", nums);
    set_at(&mut nums, 99, 999);
    println!("after out-of-bounds assign: {:?}", nums);
    set_at(&mut nums, 1, 99);
    println!("after valid assign: {:?}", nums);

    println!("demo complete!");
}
