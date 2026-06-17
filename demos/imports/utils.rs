// Reference: import semantics work
// Reference: imports
pub fn greet(name: String) -> String {
    return format!("Welcome, {}!", name);
}

pub fn format_total(items: i64, total: f64) -> String {
    return format!("{} items, total: ${}", items, total);
}
