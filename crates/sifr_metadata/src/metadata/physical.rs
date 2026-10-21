//! Versioned directory, module/name catalog and remaining payload frames.
use super::container::{ENTRY_SIZE, HEADER_SIZE, MAGIC, VERSION};
use super::physical_input::PhysicalInput;
use super::{Compatibility, Limits, Module, Record, Result, Text, err};

const PHYSICAL_HEADER: usize = HEADER_SIZE + 8;

pub(super) fn encode(logical: &[u8], limits: Limits) -> Result<Vec<u8>> {
    if logical.len() < HEADER_SIZE {
        return Err(err("truncated logical container"));
    }
    let count = u32::from_le_bytes(
        logical[12..16]
            .try_into()
            .map_err(|_| err("truncated record count"))?,
    );
    let split = directory_end(count, limits)?;
    if split > logical.len() {
        return Err(err("truncated logical directory"));
    }
    let prefix = zstd::bulk::compress(&logical[..split], 9)
        .map_err(|e| err(format!("compress directory: {e}")))?;
    let lengths = payload_lengths(&logical[..split], limits)?;
    if split
        .checked_add(lengths[0])
        .and_then(|n| n.checked_add(lengths[1]))
        != Some(logical.len())
    {
        return Err(err("payload groups differ from expanded length"));
    }
    let catalog_end = split + lengths[0];
    let catalog = zstd::bulk::compress(&logical[split..catalog_end], 9)
        .map_err(|e| err(format!("compress catalog: {e}")))?;
    let payload = zstd::bulk::compress(&logical[catalog_end..], 9)
        .map_err(|e| err(format!("compress payload: {e}")))?;
    let size = PHYSICAL_HEADER
        .checked_add(prefix.len())
        .and_then(|size| size.checked_add(catalog.len()))
        .and_then(|size| size.checked_add(payload.len()))
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
    bytes.extend_from_slice(&prefix);
    bytes.extend_from_slice(&catalog);
    bytes.extend_from_slice(&payload);
    Ok(bytes)
}

pub(super) fn open(
    input: Vec<u8>,
    expected: Compatibility,
    limits: Limits,
) -> Result<PhysicalInput> {
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
    let prefix_len = directory_end(count, limits)?;
    if expanded > limits.file_bytes || expanded < prefix_len as u64 {
        return Err(err(
            "expanded size or directory outside bounded container limits",
        ));
    }
    let expanded =
        usize::try_from(expanded).map_err(|_| err("expanded size exceeds address space"))?;
    let frames = &input[PHYSICAL_HEADER..];
    let prefix_size = frame_size(frames, prefix_len)?;
    let prefix = zstd::bulk::decompress(&frames[..prefix_size], prefix_len)
        .map_err(|e| err(format!("bounded directory decompression: {e}")))?;
    if prefix.len() != prefix_len
        || prefix[..112] != header[..112]
        || u64::from_le_bytes(
            prefix[112..120]
                .try_into()
                .map_err(|_| err("truncated directory header"))?,
        ) != expanded as u64
    {
        return Err(err(
            "expanded file length or header differs from bounded outer header",
        ));
    }
    let lengths = payload_lengths(&prefix, limits)?;
    if prefix_len
        .checked_add(lengths[0])
        .and_then(|n| n.checked_add(lengths[1]))
        != Some(expanded)
    {
        return Err(err("payload groups differ from expanded length"));
    }
    let mut cursor = PHYSICAL_HEADER + prefix_size;
    let mut ranges = Vec::with_capacity(2);
    for length in lengths {
        let remaining = input
            .get(cursor..)
            .ok_or_else(|| err("truncated payload frame"))?;
        let size = frame_size(remaining, length)?;
        if length == 0 {
            zstd::bulk::decompress(&remaining[..size], 0)
                .map_err(|e| err(format!("invalid empty payload frame: {e}")))?;
        }
        let end = cursor
            .checked_add(size)
            .ok_or_else(|| err("frame length overflow"))?;
        ranges.push(cursor..end);
        cursor = end;
    }
    if cursor != input.len() {
        return Err(err("trailing bytes or additional compressed frame"));
    }
    Ok(PhysicalInput::new(prefix, input, ranges, lengths, expanded))
}

/// Locality only: this does not defer the provider's authoritative module/name
/// validation or decode any semantic, HIR or Rust record.
pub(super) const fn payload_group(kind: u16) -> usize {
    if kind == Module::KIND || kind == Text::KIND {
        0
    } else {
        1
    }
}

fn payload_lengths(prefix: &[u8], limits: Limits) -> Result<[usize; 2]> {
    let mut lengths = [0_usize; 2];
    for entry in prefix[HEADER_SIZE..].as_chunks::<ENTRY_SIZE>().0 {
        let kind = u16::from_le_bytes(
            entry[32..34]
                .try_into()
                .map_err(|_| err("truncated record kind"))?,
        );
        let len = u64::from_le_bytes(
            entry[44..52]
                .try_into()
                .map_err(|_| err("truncated record length"))?,
        );
        if kind == 0 || kind > super::KIND_COUNT || len == 0 || len > limits.record_bytes {
            return Err(err("invalid payload kind or bounded length"));
        }
        let len = usize::try_from(len).map_err(|_| err("payload exceeds address space"))?;
        let group = &mut lengths[payload_group(kind)];
        *group = group
            .checked_add(len)
            .ok_or_else(|| err("payload group length overflow"))?;
    }
    Ok(lengths)
}

fn directory_end(count: u32, limits: Limits) -> Result<usize> {
    if count > limits.records {
        return Err(err("directory record count exceeds limit"));
    }
    let size = (HEADER_SIZE as u64)
        .checked_add(u64::from(count) * ENTRY_SIZE as u64)
        .ok_or_else(|| err("directory length overflow"))?;
    if size > limits.file_bytes {
        return Err(err("directory outside bounded container limits"));
    }
    usize::try_from(size).map_err(|_| err("directory exceeds address space"))
}

fn frame_size(bytes: &[u8], expanded: usize) -> Result<usize> {
    if zstd::zstd_safe::get_frame_content_size(bytes)
        .map_err(|e| err(format!("invalid compressed frame header: {e}")))?
        != Some(expanded as u64)
    {
        return Err(err(
            "compressed frame content length differs from bounded layout",
        ));
    }
    zstd::zstd_safe::find_frame_compressed_size(bytes)
        .map_err(|e| err(format!("invalid compressed frame: {e}")))
}

#[cfg(test)]
pub(super) fn decode(
    input: &[u8],
    expected: Compatibility,
    limits: Limits,
) -> Result<std::io::Cursor<Vec<u8>>> {
    use std::io::Read as _;
    let mut input = open(input.to_vec(), expected, limits)?;
    let mut logical = Vec::new();
    input
        .read_to_end(&mut logical)
        .map_err(|e| err(e.to_string()))?;
    Ok(std::io::Cursor::new(logical))
}

fn read_u32(header: &[u8; PHYSICAL_HEADER], start: usize) -> u32 {
    u32::from_le_bytes(std::array::from_fn(|index| header[start + index]))
}
fn read_u64(header: &[u8; PHYSICAL_HEADER], start: usize) -> u64 {
    u64::from_le_bytes(std::array::from_fn(|index| header[start + index]))
}
