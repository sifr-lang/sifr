#[must_use]
pub const fn feature_name() -> &'static str {
    "html"
}

#[must_use]
pub fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

#[must_use]
pub fn html_unescape(value: &str) -> String {
    let mut unescaped = value.to_string();
    for (from, to) in [
        ("&amp;", "&"),
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&quot;", "\""),
        ("&#x27;", "'"),
        ("&#X27;", "'"),
        ("&#39;", "'"),
        ("&#60;", "<"),
        ("&#x3C;", "<"),
        ("&#x3c;", "<"),
        ("&#X3C;", "<"),
        ("&#X3c;", "<"),
        ("&#62;", ">"),
        ("&#x3E;", ">"),
        ("&#x3e;", ">"),
        ("&#X3E;", ">"),
        ("&#X3e;", ">"),
    ] {
        unescaped = unescaped.replace(from, to);
    }
    unescaped
}
