//! UUID intrinsic lowerers for registry migration.

use crate::RustExpr;

pub(super) fn lower_uuid4(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::Ident(
        "{ use rand::Rng; let mut rng = rand::thread_rng(); let bytes: [u8; 16] = rng.gen(); format!(\"{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}\", u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]), u16::from_be_bytes([bytes[4], bytes[5]]), u16::from_be_bytes([bytes[6], bytes[7]]) & 0x0fff, (u16::from_be_bytes([bytes[8], bytes[9]]) & 0x3fff) | 0x8000, u64::from_be_bytes([0, 0, bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]])) }".to_string(),
    ))
}
