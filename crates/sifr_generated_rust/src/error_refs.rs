use std::collections::HashSet;

pub fn collect_source_builtin_error_classes(
    text: &str,
    builtin_error_classes: &[&str],
) -> HashSet<String> {
    let mut referenced = HashSet::new();
    let mut token = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            token.push(ch);
            continue;
        }
        if !token.is_empty() {
            if builtin_error_classes.contains(&token.as_str()) {
                referenced.insert(token.clone());
            }
            token.clear();
        }
    }
    if !token.is_empty() && builtin_error_classes.contains(&token.as_str()) {
        referenced.insert(token);
    }
    referenced
}
