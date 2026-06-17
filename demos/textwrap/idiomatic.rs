fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(actual, expected);
}

#[derive(Debug, Clone)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ValueError {}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn wrap(text: &str, width: usize) -> Result<Vec<String>, ValueError> {
    if width == 0 {
        return Err(ValueError::new("wrap: width must be > 0"));
    }

    let normalized = normalize_whitespace(text);
    if normalized.is_empty() {
        return Ok(Vec::new());
    }

    let mut lines = Vec::new();
    let mut current = String::new();

    for word in normalized.split(' ') {
        if current.is_empty() {
            current.push_str(word);
            continue;
        }

        if current.len() + 1 + word.len() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    Ok(lines)
}

fn fill(text: &str, width: usize) -> Result<String, ValueError> {
    wrap(text, width).map(|lines| lines.join("\n"))
}

fn dedent(text: &str) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let min_indent = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.chars()
                .take_while(|ch| *ch == ' ' || *ch == '\t')
                .count()
        })
        .min()
        .unwrap_or(0);

    lines
        .into_iter()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                line.chars().skip(min_indent).collect()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn indent(text: &str, prefix: &str) -> String {
    text.split('\n')
        .map(|line| {
            if line.trim().is_empty() {
                line.to_string()
            } else {
                format!("{prefix}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn shorten(text: &str, width: usize) -> String {
    let placeholder = "[...]";
    let normalized = normalize_whitespace(text);
    if normalized.len() < width {
        return normalized;
    }
    if width <= placeholder.len() {
        return placeholder[..width].to_string();
    }

    let available = width - placeholder.len() - 1;
    let mut current = String::new();

    for word in normalized.split(' ') {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };

        if candidate.len() > available {
            break;
        }

        current = candidate;
    }

    if current.is_empty() {
        format!("{}{}", &normalized[..available], placeholder)
    } else {
        format!("{current} {placeholder}")
    }
}

fn collect_wrap_fill_actual() -> Vec<bool> {
    let wrapped_ok = wrap("alpha\tbeta\ngamma", 10)
        .map(|lines| lines == vec!["alpha beta".to_string(), "gamma".to_string()])
        .unwrap_or(false);
    let filled_ok = fill("alpha\tbeta\ngamma", 10)
        .map(|text| text == "alpha beta\ngamma")
        .unwrap_or(false);

    vec![wrapped_ok, filled_ok]
}

fn collect_other_actual() -> Vec<bool> {
    let wrap_empty_ok = wrap("", 5).map(|lines| lines.is_empty()).unwrap_or(false);

    vec![
        dedent("  x\n  y") == "x\ny",
        indent("x\n \ny", ">> ") == ">> x\n \n>> y",
        shorten("alpha beta gamma", 16) == "alpha beta [...]",
        wrap_empty_ok,
    ]
}

fn append_all(target: &mut Vec<bool>, values: &[bool]) {
    target.extend_from_slice(values);
}

fn main() {
    let mut actual = Vec::new();
    append_all(&mut actual, &collect_wrap_fill_actual());
    append_all(&mut actual, &collect_other_actual());

    assert_bool_vector_eq(&actual, &[true, true, true, true, true, true]);
    println!("textwrap textwrap parity demo: pass");
}
