use sifr_runtime::interop::{Handle, HandleStateError, SifrIntBridge};
use std::fmt;

#[derive(Debug)]
pub struct CompiledPattern {
    regex: regex::Regex,
    source: String,
    flags: SifrIntBridge,
}

#[derive(Debug)]
pub enum CompiledPatternError {
    Invalid(regex::Error),
    Handle(HandleStateError),
}

impl fmt::Display for CompiledPatternError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(error) => error.fmt(formatter),
            Self::Handle(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for CompiledPatternError {}

impl From<regex::Error> for CompiledPatternError {
    fn from(error: regex::Error) -> Self {
        Self::Invalid(error)
    }
}

impl From<HandleStateError> for CompiledPatternError {
    fn from(error: HandleStateError) -> Self {
        Self::Handle(error)
    }
}

pub fn compile_pattern(pattern: &str) -> Result<Handle<CompiledPattern>, CompiledPatternError> {
    compile_pattern_flags(pattern, SifrIntBridge::from(0_i64))
}

pub fn compile_pattern_flags(
    pattern: &str,
    flags: SifrIntBridge,
) -> Result<Handle<CompiledPattern>, CompiledPatternError> {
    let regex = regex_with_flags(pattern, flags.clone())?;
    Ok(Handle::new(CompiledPattern {
        regex,
        source: pattern.to_string(),
        flags,
    }))
}

pub fn compiled_pattern_search(
    pattern: &Handle<CompiledPattern>,
    text: &str,
) -> Result<Option<String>, CompiledPatternError> {
    Ok(pattern
        .inner_ref()?
        .regex
        .find(text)
        .map(|matched| matched.as_str().to_string()))
}

pub fn compiled_pattern_is_match(
    pattern: &Handle<CompiledPattern>,
    text: &str,
) -> Result<bool, CompiledPatternError> {
    Ok(pattern.inner_ref()?.regex.is_match(text))
}

pub fn compiled_pattern_replace(
    pattern: &Handle<CompiledPattern>,
    replacement: &str,
    text: &str,
) -> Result<String, CompiledPatternError> {
    Ok(pattern
        .inner_ref()?
        .regex
        .replace_all(text, replacement)
        .to_string())
}

pub fn compiled_pattern_findall(
    pattern: &Handle<CompiledPattern>,
    text: &str,
) -> Result<Vec<String>, CompiledPatternError> {
    Ok(pattern
        .inner_ref()?
        .regex
        .find_iter(text)
        .map(|matched| matched.as_str().to_string())
        .collect())
}

pub fn compiled_pattern_split(
    pattern: &Handle<CompiledPattern>,
    text: &str,
) -> Result<Vec<String>, CompiledPatternError> {
    Ok(pattern
        .inner_ref()?
        .regex
        .split(text)
        .map(str::to_string)
        .collect())
}

pub fn compiled_pattern_source(
    pattern: &Handle<CompiledPattern>,
) -> Result<String, CompiledPatternError> {
    Ok(pattern.inner_ref()?.source.clone())
}

pub fn compiled_pattern_flags(
    pattern: &Handle<CompiledPattern>,
) -> Result<SifrIntBridge, CompiledPatternError> {
    Ok(pattern.inner_ref()?.flags.clone())
}

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
