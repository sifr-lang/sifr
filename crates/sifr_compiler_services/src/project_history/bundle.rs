//! Immutable publication envelope. Payloads keep a fixed offset so inherited
//! records do not require decoding another generation's history manifest.
use super::{MANIFEST_LIMIT, Manifest, RECORD_LIMIT, invalid};
use sifr_cache_storage as storage;
use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
};

const MAGIC: &[u8; 8] = b"SIFRPH03";
const HEADER_BYTES: u64 = 16;

fn open(path: &Path) -> io::Result<(File, u64)> {
    let parent = path
        .parent()
        .ok_or_else(|| invalid("missing bundle parent"))?;
    let name = path
        .file_name()
        .ok_or_else(|| invalid("missing bundle name"))?;
    storage::payload(parent, Path::new(name))?;
    let mut file = storage::read_private_file(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.len() > HEADER_BYTES + RECORD_LIMIT + MANIFEST_LIMIT {
        return Err(invalid("project bundle size limit"));
    }
    let mut header = [0; HEADER_BYTES as usize];
    file.read_exact(&mut header)?;
    if &header[..8] != MAGIC {
        return Err(invalid("invalid project bundle format"));
    }
    let mut length = [0; 8];
    length.copy_from_slice(&header[8..]);
    let length = u64::from_le_bytes(length);
    if length > RECORD_LIMIT || metadata.len() < HEADER_BYTES + length {
        return Err(invalid("invalid project bundle payload length"));
    }
    Ok((file, length))
}

pub(super) fn manifest(path: &Path) -> io::Result<Manifest> {
    let (mut file, length) = open(path)?;
    file.seek(SeekFrom::Start(HEADER_BYTES + length))?;
    let mut bytes = Vec::new();
    file.take(MANIFEST_LIMIT + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MANIFEST_LIMIT {
        return Err(invalid("project bundle manifest grew past limit"));
    }
    let manifest: Manifest = serde_json::from_slice(&bytes)?;
    let own = manifest
        .records
        .last()
        .and_then(|id| manifest.payloads.get(id));
    if !own.is_some_and(|payload| payload.owner.is_none() && payload.length == length) {
        return Err(invalid("project bundle payload does not match manifest"));
    }
    Ok(manifest)
}

pub(super) fn payload(path: &Path, expected: u64) -> io::Result<Vec<u8>> {
    let (mut file, length) = open(path)?;
    if length != expected {
        return Err(invalid("changed immutable payload length"));
    }
    let size = usize::try_from(length).map_err(|_| invalid("payload exceeds address space"))?;
    let mut bytes = vec![0; size];
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}

pub(super) fn write(path: &Path, manifest: &Manifest, bytes: &[u8]) -> io::Result<()> {
    let metadata = serde_json::to_vec(manifest)?;
    if bytes.len() as u64 > RECORD_LIMIT || metadata.len() as u64 > MANIFEST_LIMIT {
        return Err(invalid("project bundle size limit"));
    }
    let mut file = storage::new_private_file(path)?;
    file.write_all(MAGIC)?;
    file.write_all(&(bytes.len() as u64).to_le_bytes())?;
    file.write_all(bytes)?;
    file.write_all(&metadata)?;
    // The record and its complete index cross the same durability boundary.
    file.sync_all()
}
