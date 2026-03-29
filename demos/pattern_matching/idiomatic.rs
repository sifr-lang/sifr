#[derive(Debug, Clone)]
enum IntOrStr {
    Int(i64),
    Str(String),
}

fn describe_number(value: i64) -> &'static str {
    match value {
        0 => "zero",
        1 => "one",
        2 => "two",
        _ => "many",
    }
}

fn classify_http(method: &str) -> &'static str {
    match method {
        "GET" | "HEAD" => "read",
        "POST" | "PUT" | "PATCH" => "write",
        "DELETE" => "delete",
        _ => "other",
    }
}

fn classify_score(score: i64) -> &'static str {
    match score {
        90.. => "A",
        80..=89 => "B",
        70..=79 => "C",
        60..=69 => "D",
        _ => "F",
    }
}

fn describe_optional(value: Option<i64>) -> &'static str {
    match value {
        None => "nothing",
        Some(_) => "something",
    }
}

fn describe_union(value: &IntOrStr) -> &'static str {
    match value {
        IntOrStr::Int(_) => "integer",
        IntOrStr::Str(_) => "string",
    }
}

fn make_int_union() -> IntOrStr {
    IntOrStr::Int(42)
}

fn make_str_union() -> IntOrStr {
    IntOrStr::Str("hello".to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Point {
    x: i64,
    y: i64,
}

fn classify_point(point: Point) -> &'static str {
    match point {
        Point { x: 0, y: 0 } => "origin",
        Point { y: 0, .. } => "on x-axis",
        Point { x: 0, .. } => "on y-axis",
        Point { .. } => "general",
    }
}

fn classify_pair(pair: (i64, i64)) -> &'static str {
    match pair {
        (0, 0) => "origin",
        (_, 0) => "x-axis",
        (0, _) => "y-axis",
        _ => "general",
    }
}

fn classify_quadrant(point: Point) -> &'static str {
    match point {
        Point { x: 0, y: 0 } => "origin",
        Point { x, y } if x > 0 && y > 0 => "Q1",
        Point { x, y } if x < 0 && y > 0 => "Q2",
        _ => "other",
    }
}

fn main() {
    println!("=== Literal Patterns ===");
    println!("{}", describe_number(0));
    println!("{}", describe_number(1));
    println!("{}", describe_number(42));

    println!("=== OR Patterns ===");
    println!("{}", classify_http("GET"));
    println!("{}", classify_http("POST"));
    println!("{}", classify_http("DELETE"));
    println!("{}", classify_http("OPTIONS"));

    println!("=== Guard Patterns ===");
    println!("{}", classify_score(95));
    println!("{}", classify_score(85));
    println!("{}", classify_score(55));

    println!("=== Optional Matching ===");
    println!("{}", describe_optional(None));
    println!("{}", describe_optional(Some(42)));

    println!("=== Union Matching ===");
    let a = make_int_union();
    let b = make_str_union();
    if let IntOrStr::Int(value) = &a {
        let _ = value;
    }
    if let IntOrStr::Str(value) = &b {
        let _ = value;
    }
    println!("{}", describe_union(&a));
    println!("{}", describe_union(&b));

    println!("=== Class Destructuring ===");
    println!("{}", classify_point(Point { x: 0, y: 0 }));
    println!("{}", classify_point(Point { x: 3, y: 0 }));
    println!("{}", classify_point(Point { x: 0, y: 4 }));
    println!("{}", classify_point(Point { x: 3, y: 4 }));

    println!("=== Tuple Patterns ===");
    println!("{}", classify_pair((0, 0)));
    println!("{}", classify_pair((3, 0)));
    println!("{}", classify_pair((0, 4)));
    println!("{}", classify_pair((3, 4)));

    println!("=== Nested Patterns ===");
    println!("{}", classify_quadrant(Point { x: 0, y: 0 }));
    println!("{}", classify_quadrant(Point { x: 3, y: 4 }));
    println!("{}", classify_quadrant(Point { x: -2, y: 5 }));
    println!("{}", classify_quadrant(Point { x: -1, y: -1 }));
}
