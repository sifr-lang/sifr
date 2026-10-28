//! Immutable filenames are the commit index. Discovery keeps one head in memory;
//! its cost follows unpruned directory entries, not retained semantic records.
use super::{invalid, key};
use std::{fs, io, path::Path};

pub fn generation_sequence(name: &str) -> Option<u64> {
    let (sequence, digest) = name.split_once('-')?;
    if sequence.len() != 16
        || !sequence
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || !key(digest)
    {
        return None;
    }
    u64::from_str_radix(sequence, 16).ok()
}

pub fn generation_key(name: &str) -> bool {
    generation_sequence(name).is_some()
}

pub fn latest_generation(parent: &Path) -> io::Result<Option<String>> {
    sifr_cache_storage::check_owned(parent)?;
    let mut latest: Option<(u64, String)> = None;
    let mut ambiguous = false;
    for entry in fs::read_dir(parent)? {
        let name = entry?.file_name();
        let Some(name) = name.to_str() else { continue };
        let Some(sequence) = generation_sequence(name) else {
            continue;
        };
        match &latest {
            Some((current, _)) if *current > sequence => {}
            Some((current, _)) if *current == sequence => ambiguous = true,
            _ => {
                latest = Some((sequence, name.to_owned()));
                ambiguous = false;
            }
        }
    }
    if ambiguous {
        return Err(invalid("ambiguous project commit head"));
    }
    Ok(latest.map(|(_, name)| name))
}

pub fn next_sequence(head: Option<&str>) -> io::Result<u64> {
    match head {
        None => Ok(0),
        Some(name) => generation_sequence(name)
            .and_then(|value| value.checked_add(1))
            .ok_or_else(|| invalid("project commit sequence exhausted or invalid")),
    }
}
