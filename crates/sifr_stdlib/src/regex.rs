use sifr_runtime::interop::SifrIntBridge;

pub fn re_match(pattern: &str, text: &str) -> Result<bool, regex::Error> {
    regex::Regex::new(pattern).map(|re| re.is_match(text))
}

pub fn re_find(pattern: &str, text: &str) -> Result<Option<String>, regex::Error> {
    regex::Regex::new(pattern).map(|re| re.find(text).map(|matched| matched.as_str().to_string()))
}

pub fn re_replace(pattern: &str, replacement: &str, text: &str) -> Result<String, regex::Error> {
    regex::Regex::new(pattern).map(|re| re.replace_all(text, replacement).to_string())
}

pub fn re_findall(pattern: &str, text: &str) -> Result<Vec<String>, regex::Error> {
    regex::Regex::new(pattern).map(|re| {
        re.find_iter(text)
            .map(|matched| matched.as_str().to_string())
            .collect()
    })
}

pub fn re_split(pattern: &str, text: &str) -> Result<Vec<String>, regex::Error> {
    regex::Regex::new(pattern).map(|re| re.split(text).map(str::to_string).collect())
}

pub fn re_find_start(pattern: &str, text: &str) -> Result<SifrIntBridge, regex::Error> {
    regex::Regex::new(pattern).map(|re| SifrIntBridge::from(match_start(&re, text)))
}

pub fn re_find_end(pattern: &str, text: &str) -> Result<SifrIntBridge, regex::Error> {
    regex::Regex::new(pattern).map(|re| SifrIntBridge::from(match_end(&re, text)))
}

pub fn re_match_flags(
    pattern: &str,
    text: &str,
    flags: SifrIntBridge,
) -> Result<bool, regex::Error> {
    regex_with_flags(pattern, flags).map(|re| re.is_match(text))
}

pub fn re_find_flags(
    pattern: &str,
    text: &str,
    flags: SifrIntBridge,
) -> Result<Option<String>, regex::Error> {
    regex_with_flags(pattern, flags)
        .map(|re| re.find(text).map(|matched| matched.as_str().to_string()))
}

pub fn re_replace_flags(
    pattern: &str,
    replacement: &str,
    text: &str,
    flags: SifrIntBridge,
) -> Result<String, regex::Error> {
    regex_with_flags(pattern, flags).map(|re| re.replace_all(text, replacement).to_string())
}

pub fn re_findall_flags(
    pattern: &str,
    text: &str,
    flags: SifrIntBridge,
) -> Result<Vec<String>, regex::Error> {
    regex_with_flags(pattern, flags).map(|re| {
        re.find_iter(text)
            .map(|matched| matched.as_str().to_string())
            .collect()
    })
}

pub fn re_split_flags(
    pattern: &str,
    text: &str,
    flags: SifrIntBridge,
) -> Result<Vec<String>, regex::Error> {
    regex_with_flags(pattern, flags).map(|re| re.split(text).map(str::to_string).collect())
}

fn regex_with_flags(pattern: &str, flags: SifrIntBridge) -> Result<regex::Regex, regex::Error> {
    let flags = flags.to_i64_saturating();
    let mut prefixed = String::new();
    if flags & 2 != 0 {
        prefixed.push_str("(?i)");
    }
    if flags & 8 != 0 {
        prefixed.push_str("(?m)");
    }
    if flags & 16 != 0 {
        prefixed.push_str("(?s)");
    }
    if flags & 64 != 0 {
        prefixed.push_str("(?x)");
    }
    prefixed.push_str(pattern);
    regex::Regex::new(&prefixed)
}

fn match_start(re: &regex::Regex, text: &str) -> i64 {
    re.find(text)
        .map_or(-1, |matched| usize_to_i64_saturating(matched.start()))
}

fn match_end(re: &regex::Regex, text: &str) -> i64 {
    re.find(text)
        .map_or(-1, |matched| usize_to_i64_saturating(matched.end()))
}

fn usize_to_i64_saturating(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}
