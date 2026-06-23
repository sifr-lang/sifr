pub const LEAF_FEATURES: &[&str] = &[
    "json",
    "regex",
    "uuid",
    "hash",
    "base64",
    "html",
    "calendar",
    "platform",
    "toml",
    "url",
    "gzip",
    "zipfile",
    "unicode",
    "i18n",
    "net",
    "tls",
    "http",
    "python",
    "process",
    "fs",
    "signals",
    "runtime-observability",
];

pub const UMBRELLA_FEATURES: &[&str] = &["text-data", "network-stack"];

#[must_use]
pub fn leaf_features() -> &'static [&'static str] {
    LEAF_FEATURES
}

#[must_use]
pub fn umbrella_features() -> &'static [&'static str] {
    UMBRELLA_FEATURES
}
