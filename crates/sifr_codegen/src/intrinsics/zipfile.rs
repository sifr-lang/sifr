//! Zipfile intrinsic lowerers for registry migration.

use crate::RustExpr;

fn borrowed_str(expr: &str) -> String {
    format!("&({expr})")
}

pub(super) fn lower_zip_create(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "(|| -> Result<(), IOError> {{ let __f = std::fs::File::create({}).map_err(__io_err)?; drop(zip::ZipWriter::new(__f)); Ok(()) }})()",
        borrowed_str(&args[0])
    )))
}

pub(super) fn lower_zip_add_file(args: &[String]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "(|| -> Result<(), IOError> {{ let __path = {}; let __name = {}; let __content = {}; let __f = std::fs::OpenOptions::new().read(true).write(true).open(__path).map_err(__io_err)?; let mut __zip = zip::ZipWriter::new_append(__f).map_err(|e| IOError::new(e.to_string()))?; let __opts = zip::write::FileOptions::default(); __zip.start_file((__name).to_string(), __opts).map_err(|e| IOError::new(e.to_string()))?; use std::io::Write; __zip.write_all(__content.as_bytes()).map_err(__io_err)?; __zip.finish().map_err(|e| IOError::new(e.to_string()))?; Ok(()) }})()",
        borrowed_str(&args[0]),
        borrowed_str(&args[1]),
        borrowed_str(&args[2])
    )))
}

pub(super) fn lower_zip_read_file(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "(|| -> Result<String, IOError> {{ let __f = std::fs::File::open({}).map_err(__io_err)?; let mut __zip = zip::ZipArchive::new(__f).map_err(|e| IOError::new(e.to_string()))?; let mut __file = __zip.by_name({}).map_err(|e| IOError::new(e.to_string()))?; let mut __content = String::new(); use std::io::Read; __file.read_to_string(&mut __content).map_err(__io_err)?; Ok(__content) }})()",
        borrowed_str(&args[0]),
        borrowed_str(&args[1])
    )))
}

pub(super) fn lower_zip_namelist(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Ident(format!(
        "(|| -> Result<Vec<String>, IOError> {{ let __f = std::fs::File::open({}).map_err(__io_err)?; let mut __zip = zip::ZipArchive::new(__f).map_err(|e| IOError::new(e.to_string()))?; Ok((0..__zip.len()).map(|i| __zip.by_index(i).map(|f| f.name().to_string()).unwrap_or_default()).collect()) }})()",
        borrowed_str(&args[0])
    )))
}
