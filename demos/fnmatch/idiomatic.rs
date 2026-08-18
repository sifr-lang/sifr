fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(actual, expected);
}

fn wildcard_match(name: &str, pattern: &str) -> bool {
    let name: Vec<char> = name.chars().collect();
    let pattern: Vec<char> = pattern.chars().collect();
    let (mut ni, mut pi) = (0usize, 0usize);
    let mut star_pattern = None;
    let mut star_name = 0usize;

    while ni < name.len() {
        if pi < pattern.len() && (pattern[pi] == '?' || pattern[pi] == name[ni]) {
            ni += 1;
            pi += 1;
        } else if pi < pattern.len() && pattern[pi] == '*' {
            star_pattern = Some(pi);
            pi += 1;
            star_name = ni;
        } else if let Some(star_index) = star_pattern {
            pi = star_index + 1;
            star_name += 1;
            ni = star_name;
        } else {
            return false;
        }
    }

    while pi < pattern.len() && pattern[pi] == '*' {
        pi += 1;
    }

    pi == pattern.len()
}

fn fnmatch(name: &str, pattern: &str) -> bool {
    wildcard_match(name, pattern)
}

fn filter(names: &[String], pattern: &str) -> Vec<String> {
    names
        .iter()
        .filter(|name| fnmatch(name, pattern))
        .cloned()
        .collect()
}

fn collect_match_actual() -> Vec<bool> {
    vec![
        fnmatch("hello.txt", "*.txt"),
        !fnmatch("hello.py", "*.txt"),
        fnmatch("abc", "a?c"),
        !fnmatch("AbC", "abc"),
    ]
}

fn collect_filter_actual() -> Vec<bool> {
    let names = vec![
        "main.py".to_string(),
        "notes.txt".to_string(),
        "lib.py".to_string(),
    ];

    vec![
        filter(&names, "*.py") == vec!["main.py".to_string(), "lib.py".to_string()],
        filter(&names, "README*").is_empty(),
    ]
}

fn append_all(target: &mut Vec<bool>, values: &[bool]) {
    target.extend_from_slice(values);
}

fn main() {
    let mut actual = Vec::new();
    append_all(&mut actual, &collect_match_actual());
    append_all(&mut actual, &collect_filter_actual());

    assert_bool_vector_eq(&actual, &[true, true, true, true, true, true]);
    println!("fnmatch fnmatch parity demo: pass");
}
