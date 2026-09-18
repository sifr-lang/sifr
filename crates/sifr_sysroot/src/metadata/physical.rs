//! Versioned physical compression only: record identities and lazy decoding are unchanged.
use super::container::{ENTRY_SIZE, HEADER_SIZE, MAGIC, VERSION};
use super::{Compatibility, Limits, Result, err};
use std::io::Cursor;

const PHYSICAL_HEADER: usize = HEADER_SIZE + 8;

pub(super) fn encode(logical: &[u8], limits: Limits) -> Result<Vec<u8>> {
    if logical.len() < HEADER_SIZE {
        return Err(err("truncated logical container"));
    }
    let compressed =
        zstd::bulk::compress(logical, 9).map_err(|e| err(format!("compress container: {e}")))?;
    let size = PHYSICAL_HEADER
        .checked_add(compressed.len())
        .ok_or_else(|| err("physical length overflow"))?;
    if size as u64 > limits.file_bytes || logical.len() as u64 > limits.file_bytes {
        return Err(err(
            "physical or expanded container exceeds file byte limit",
        ));
    }
    let mut bytes = Vec::with_capacity(size);
    bytes.extend_from_slice(&logical[..112]);
    bytes.extend_from_slice(&(size as u64).to_le_bytes());
    bytes.extend_from_slice(&(logical.len() as u64).to_le_bytes());
    bytes.extend_from_slice(&compressed);
    Ok(bytes)
}

pub(super) fn decode(
    input: &[u8],
    expected: Compatibility,
    limits: Limits,
) -> Result<Cursor<Vec<u8>>> {
    let size = input.len() as u64;
    if size < PHYSICAL_HEADER as u64 || size > limits.file_bytes {
        return Err(err("physical size outside bounded container limits"));
    }
    let header: &[u8; PHYSICAL_HEADER] = input[..PHYSICAL_HEADER]
        .try_into()
        .map_err(|_| err("truncated physical header"))?;
    if &header[..8] != MAGIC || read_u32(header, 8) != VERSION {
        return Err(err("unknown magic or incompatible schema version"));
    }
    if header[16..48] != expected.compiler
        || header[48..80] != expected.semantic_target
        || header[80..112] != expected.stdlib_inputs
    {
        return Err(err(
            "compiler, semantic target or stdlib input identity mismatch",
        ));
    }
    if read_u64(header, 112) != size {
        return Err(err("declared physical file length differs from input"));
    }
    let expanded = read_u64(header, 120);
    let count = read_u32(header, 12);
    if expanded > limits.file_bytes
        || expanded < HEADER_SIZE as u64
        || count > limits.records
        || HEADER_SIZE as u64 + u64::from(count) * ENTRY_SIZE as u64 > expanded
    {
        return Err(err(
            "expanded size or directory outside bounded container limits",
        ));
    }
    let payload = &input[PHYSICAL_HEADER..];
    if zstd::zstd_safe::find_frame_compressed_size(payload)
        .map_err(|e| err(format!("invalid compressed frame: {e}")))?
        != payload.len()
    {
        return Err(err("trailing bytes or additional compressed frame"));
    }
    let expanded =
        usize::try_from(expanded).map_err(|_| err("expanded size exceeds address space"))?;
    let logical = zstd::bulk::decompress(payload, expanded)
        .map_err(|e| err(format!("bounded container decompression: {e}")))?;
    if logical.len() != expanded || logical[..112] != header[..112] {
        return Err(err(
            "expanded file length or header differs from bounded outer header",
        ));
    }
    Ok(Cursor::new(logical))
}

fn read_u32(header: &[u8; PHYSICAL_HEADER], start: usize) -> u32 {
    u32::from_le_bytes([
        header[start],
        header[start + 1],
        header[start + 2],
        header[start + 3],
    ])
}
fn read_u64(header: &[u8; PHYSICAL_HEADER], start: usize) -> u64 {
    u64::from_le_bytes(std::array::from_fn(|index| header[start + index]))
}
