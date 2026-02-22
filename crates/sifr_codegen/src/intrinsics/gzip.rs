//! `GZip` intrinsic lowerers for registry migration.

use crate::RustExpr;

fn borrowed_str(expr: &str) -> String {
    format!("&({expr})")
}

pub(super) fn lower_gzip_compress(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ use std::io::Write; let __data = {}.as_bytes(); let mut __enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default()); __enc.write_all(__data).unwrap_or(()); __enc.finish().unwrap_or_default().iter().map(|b| *b as i64).collect::<Vec<i64>>() }}",
        borrowed_str(&args[0])
    )))
}

pub(super) fn lower_gzip_decompress(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "(|| -> Result<String, IOError> {{ use std::io::Read; let __bytes: Vec<u8> = {}.iter().map(|b| *b as u8).collect(); let mut __dec = flate2::read::GzDecoder::new(__bytes.as_slice()); let mut __out = String::new(); __dec.read_to_string(&mut __out).map_err(__io_err)?; Ok(__out) }})()",
        args[0]
    )))
}
