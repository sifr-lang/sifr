use crate::{intrinsics, methods, RustEmitter};
use sifr_hir::HirExpr;
use sifr_type_system::Type;

impl RustEmitter {
    /// Check if a name is a stdlib constant.
    pub(crate) fn is_stdlib_constant(&self, name: &str) -> bool {
        matches!(name, "pi" | "e" | "tau" | "inf" | "nan")
            && self.intrinsic_functions.contains(name)
    }

    /// Emit a stdlib constant value.
    pub(crate) fn emit_stdlib_constant(&mut self, name: &str) {
        match name {
            "pi" => self.write("std::f64::consts::PI"),
            "e" => self.write("std::f64::consts::E"),
            "tau" => self.write("std::f64::consts::TAU"),
            "inf" => self.write("f64::INFINITY"),
            "nan" => self.write("f64::NAN"),
            _ => self.write(name),
        }
    }

    /// Emit an intrinsic function call with the correct Rust code.
    pub(crate) fn emit_intrinsic_call(&mut self, func: &str, args: &[HirExpr]) {
        if self.try_emit_intrinsic_via_registry(func, args) {
            return;
        }

        match func {
            // sifr.io
            "read_text" => {
                self.write("std::fs::read_to_string(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map_err(__io_err)");
            }
            "write_text" => {
                self.write("std::fs::write(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(", ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").map(|_| ()).map_err(__io_err)");
            }
            "exists" => {
                self.write("std::path::Path::new(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").exists()");
            }
            "read_lines" => {
                self.write("std::fs::read_to_string(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|s| s.lines().map(|l| l.to_string()).collect::<Vec<String>>()).map_err(__io_err)");
            }
            "append_text" => {
                self.write("{ use std::io::Write; (|| -> Result<(), IOError> { let mut _f = std::fs::OpenOptions::new().append(true).create(true).open(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map_err(__io_err)?; write!(_f, \"{}\", ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").map_err(__io_err)?; Ok(()) })() }");
            }
            "getcwd" => {
                self.write("std::env::current_dir().map(|p| p.to_string_lossy().to_string()).map_err(__io_err)");
            }
            "listdir" => {
                self.write("std::fs::read_dir(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|rd| rd.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().to_string()).collect::<Vec<String>>()).map_err(__io_err)");
            }
            "mkdir" => {
                self.write("std::fs::create_dir_all(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|_| ()).map_err(__io_err)");
            }
            "rmdir" => {
                self.write("std::fs::remove_dir(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|_| ()).map_err(__io_err)");
            }
            "remove_file" => {
                self.write("std::fs::remove_file(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|_| ()).map_err(__io_err)");
            }
            "rename" => {
                self.write("std::fs::rename(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(", ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").map(|_| ()).map_err(__io_err)");
            }
            "is_file" => {
                self.write("std::path::Path::new(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").is_file()");
            }
            "is_dir" => {
                self.write("std::path::Path::new(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").is_dir()");
            }
            "copy_file" => {
                self.write("std::fs::copy(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(", ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").map(|_| ()).map_err(__io_err)");
            }
            "walk_dir" => {
                self.write("{ fn __walk(p: &std::path::Path) -> Result<Vec<String>, IOError> { let mut r = Vec::new(); let entries = std::fs::read_dir(p).map_err(__io_err)?; for e in entries { let e = e.map_err(__io_err)?; let path = e.path(); r.push(path.display().to_string()); if path.is_dir() { r.extend(__walk(&path)?); } } Ok(r) } __walk(std::path::Path::new(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(")) }");
            }
            "rmdir_all" => {
                self.write("std::fs::remove_dir_all(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|_| ()).map_err(__io_err)");
            }
            "gettempdir" => {
                self.write("std::env::temp_dir().display().to_string()");
            }
            "makedirs" => {
                self.write("std::fs::create_dir_all(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|_| ()).map_err(__io_err)");
            }
            // sifr.json
            "json_loads" => {
                self.write("serde_json::from_str::<serde_json::Value>(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|v| v.to_string()).map_err(|e| JSONDecodeError { message: e.to_string(), line: e.line() as i64, column: e.column() as i64 })");
            }
            "json_dumps" => {
                self.write("serde_json::to_string(&");
                self.emit_expr(&args[0]);
                self.write(").unwrap_or_default()");
            }
            // sifr.env
            "env_get" => {
                self.write("{ let __k = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; if __k.is_empty() || __k.contains('=') || __k.as_bytes().contains(&0) { None } else { std::env::var(__k).ok() } }");
            }
            "env_set" => {
                self.write("{ let __k = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __v = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; if !__k.is_empty() && !__k.contains('=') && !__k.as_bytes().contains(&0) && !__v.as_bytes().contains(&0) { std::env::set_var(__k, __v); } }");
            }
            "env_unset" => {
                self.write("{ let __k = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; if !__k.is_empty() && !__k.contains('=') && !__k.as_bytes().contains(&0) { std::env::remove_var(__k); } }");
            }
            "env_keys" => {
                self.write("std::env::vars_os().map(|(k, _)| k.to_string_lossy().to_string()).collect::<Vec<String>>()");
            }
            "env_values" => {
                self.write("std::env::vars_os().map(|(_, v)| v.to_string_lossy().to_string()).collect::<Vec<String>>()");
            }
            "env_items" => {
                self.write("std::env::vars_os().map(|(k, v)| format!(\"{}={}\", k.to_string_lossy(), v.to_string_lossy())).collect::<Vec<String>>()");
            }
            // sifr.os
            "run_command" => {
                self.write("(|| -> Result<String, IOError> { let output = std::process::Command::new(\"sh\").args([\"-c\", ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("]).output().map_err(__io_err)?; Ok(String::from_utf8_lossy(&output.stdout).trim().to_string()) })()");
            }
            "get_args" => {
                self.write("std::env::args().collect::<Vec<String>>()");
            }
            // sifr.math
            "sqrt" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").sqrt()");
            }
            "floor" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").floor() as i64");
            }
            "ceil" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").ceil() as i64");
            }
            "abs_val" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").abs()");
            }
            "log" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").ln()");
            }
            "cbrt" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").cbrt()");
            }
            "exp2" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").exp2()");
            }
            "sin" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").sin()");
            }
            "cos" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").cos()");
            }
            "tan" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").tan()");
            }
            "pow_val" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").powf(");
                self.emit_expr(&args[1]);
                self.write(")");
            }
            "min_val" => {
                self.write("{ let __a = ");
                self.emit_expr(&args[0]);
                self.write("; let __b = ");
                self.emit_expr(&args[1]);
                self.write("; if __a < __b { __a } else { __b } }");
            }
            "max_val" => {
                self.write("{ let __a = ");
                self.emit_expr(&args[0]);
                self.write("; let __b = ");
                self.emit_expr(&args[1]);
                self.write("; if __a > __b { __a } else { __b } }");
            }
            "round_val" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").round() as i64");
            }
            "asin" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").asin()");
            }
            "acos" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").acos()");
            }
            "atan" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").atan()");
            }
            "atan2" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").atan2(");
                self.emit_expr(&args[1]);
                self.write(")");
            }
            "sinh" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").sinh()");
            }
            "cosh" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").cosh()");
            }
            "tanh" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").tanh()");
            }
            "log10" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").log10()");
            }
            "log2" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").log2()");
            }
            "degrees" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").to_degrees()");
            }
            "radians" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").to_radians()");
            }
            "isnan" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").is_nan()");
            }
            "isinf" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").is_infinite()");
            }
            "trunc" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").trunc() as i64");
            }
            "copysign" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").copysign(");
                self.emit_expr(&args[1]);
                self.write(")");
            }
            "signbit" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").is_sign_negative()");
            }
            "fmod" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(") % (");
                self.emit_expr(&args[1]);
                self.write(")");
            }
            "remainder" => {
                self.write("{ let __x: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; let __y: f64 = ");
                self.emit_expr(&args[1]);
                self.write("; if __x.is_nan() || __y.is_nan() { f64::NAN } else if __y == 0.0 || __x.is_infinite() { f64::NAN } else if __y.is_infinite() { __x } else { let __q = __x / __y; let __n0 = __q.trunc(); let __frac = __q - __n0; let __abs_frac = __frac.abs(); let __n = if __abs_frac < 0.5 { __n0 } else if __abs_frac > 0.5 { __n0 + __q.signum() } else if (__n0 as i64) % 2 == 0 { __n0 } else { __n0 + __q.signum() }; let __r = __x - __n * __y; if __r == 0.0 { 0.0f64.copysign(__x) } else { __r } } }");
            }
            "hypot" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").hypot(");
                self.emit_expr(&args[1]);
                self.write(")");
            }
            "fma" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").mul_add(");
                self.emit_expr(&args[1]);
                self.write(", ");
                self.emit_expr(&args[2]);
                self.write(")");
            }
            "fmax" => {
                self.write("{ let __a: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; let __b: f64 = ");
                self.emit_expr(&args[1]);
                self.write("; __a.max(__b) }");
            }
            "fmin" => {
                self.write("{ let __a: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; let __b: f64 = ");
                self.emit_expr(&args[1]);
                self.write("; __a.min(__b) }");
            }
            "exp" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").exp()");
            }
            "expm1" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").exp_m1()");
            }
            "log1p" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").ln_1p()");
            }
            "fabs" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").abs()");
            }
            "isfinite" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").is_finite()");
            }
            "isnormal" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").is_normal()");
            }
            "issubnormal" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").is_subnormal()");
            }
            "acosh" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").acosh()");
            }
            "asinh" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").asinh()");
            }
            "atanh" => {
                self.write("(");
                self.emit_expr(&args[0]);
                self.write(").atanh()");
            }
            "isqrt" => {
                self.write("{ let __n = ");
                self.emit_expr(&args[0]);
                self.write(" as f64; __n.sqrt() as i64 }");
            }
            "dist" => {
                self.write("{ let __p = &");
                self.emit_expr(&args[0]);
                self.write("; let __q = &");
                self.emit_expr(&args[1]);
                self.write("; if __p.len() != __q.len() { f64::NAN } else if __p.is_empty() { 0.0 } else { let mut __scale = 0.0f64; let mut __ssq = 1.0f64; for __i in 0..__p.len() { let __d = (__p[__i] - __q[__i]).abs(); if __d != 0.0 { if __scale < __d { let __r = __scale / __d; __ssq = 1.0 + __ssq * __r * __r; __scale = __d; } else { let __r = __d / __scale; __ssq += __r * __r; } } } if __scale == 0.0 { 0.0 } else { __scale * __ssq.sqrt() } } }");
            }
            "fsum" => {
                self.write("{ let __data = &");
                self.emit_expr(&args[0]);
                self.write("; let mut __sum = 0.0f64; let mut __comp = 0.0f64; let mut __pos_inf = false; let mut __neg_inf = false; let mut __has_nan = false; for __x in __data.iter() { let __v = *__x; if __v.is_nan() { __has_nan = true; continue; } if __v.is_infinite() { if __v.is_sign_positive() { __pos_inf = true; } else { __neg_inf = true; } continue; } let __t = __sum + __v; if __sum.abs() >= __v.abs() { __comp += (__sum - __t) + __v; } else { __comp += (__v - __t) + __sum; } __sum = __t; } if __has_nan || (__pos_inf && __neg_inf) { f64::NAN } else if __pos_inf { f64::INFINITY } else if __neg_inf { f64::NEG_INFINITY } else { __sum + __comp } }");
            }
            "sumprod" => {
                self.write("{ let __p = &");
                self.emit_expr(&args[0]);
                self.write("; let __q = &");
                self.emit_expr(&args[1]);
                self.write("; let __len = __p.len().min(__q.len()); let mut __sum = 0.0f64; for __i in 0..__len { __sum += __p[__i] * __q[__i]; } __sum }");
            }
            // sifr.test
            "assert_eq" => {
                self.write("assert_eq!(");
                self.emit_expr(&args[0]);
                self.write(", ");
                self.emit_expr(&args[1]);
                self.write(")");
            }
            "assert_ne" => {
                self.write("assert_ne!(");
                self.emit_expr(&args[0]);
                self.write(", ");
                self.emit_expr(&args[1]);
                self.write(")");
            }
            "assert_true" => {
                self.write("assert!(");
                self.emit_expr(&args[0]);
                self.write(")");
            }
            "assert_false" => {
                self.write("assert!(!(");
                self.emit_expr(&args[0]);
                self.write("))");
            }
            "assert_almost_eq" => {
                self.write("assert!((");
                self.emit_expr(&args[0]);
                self.write(" - (");
                self.emit_expr(&args[1]);
                self.write(")).abs() < ");
                self.emit_expr(&args[2]);
                self.write(", \"assert_almost_eq failed: {} != {} (tolerance {})\", ");
                self.emit_expr(&args[0]);
                self.write(", ");
                self.emit_expr(&args[1]);
                self.write(", ");
                self.emit_expr(&args[2]);
                self.write(")");
            }
            "assert_gt" => {
                self.write("assert!(");
                self.emit_expr(&args[0]);
                self.write(" > ");
                self.emit_expr(&args[1]);
                self.write(", \"assert_gt failed: {} is not > {}\", ");
                self.emit_expr(&args[0]);
                self.write(", ");
                self.emit_expr(&args[1]);
                self.write(")");
            }
            "assert_lt" => {
                self.write("assert!(");
                self.emit_expr(&args[0]);
                self.write(" < ");
                self.emit_expr(&args[1]);
                self.write(", \"assert_lt failed: {} is not < {}\", ");
                self.emit_expr(&args[0]);
                self.write(", ");
                self.emit_expr(&args[1]);
                self.write(")");
            }
            // sifr.collections — Set operations
            "new_set" => {
                self.write("Vec::<i64>::new()");
            }
            "set_from_list" => {
                self.write("{ let mut s = ");
                self.emit_collection_expr(&args[0]);
                self.write("; s.sort(); s.dedup(); s }");
            }
            "set_add" => {
                self.write("{ let mut s = ");
                self.emit_collection_expr(&args[0]);
                self.write("; let v = ");
                self.emit_expr(&args[1]);
                self.write("; if !s.contains(&v) { s.push(v); } s }");
            }
            "set_contains" => {
                self.emit_expr(&args[0]);
                self.write(".contains(&");
                self.emit_expr(&args[1]);
                self.write(")");
            }
            "set_remove" => {
                self.write("{ let mut s = ");
                self.emit_collection_expr(&args[0]);
                self.write("; s.retain(|x| *x != ");
                self.emit_expr(&args[1]);
                self.write("); s }");
            }
            "set_len" => {
                self.emit_expr(&args[0]);
                self.write(".len() as i64");
            }
            "set_union" => {
                self.write("{ let mut s = ");
                self.emit_collection_expr(&args[0]);
                self.write("; for v in ");
                self.emit_expr(&args[1]);
                self.write(".iter() { if !s.contains(v) { s.push(*v); } } s.sort(); s }");
            }
            "set_intersection" => {
                self.write("{ let __a = ");
                self.emit_collection_expr(&args[0]);
                self.write("; let __b = ");
                self.emit_collection_expr(&args[1]);
                self.write(
                    "; __a.iter().filter(|x| __b.contains(x)).cloned().collect::<Vec<i64>>() }",
                );
            }
            // sifr.collections — Counter
            "counter_from_list" => {
                self.write("{ let mut counts = std::collections::HashMap::<String, i64>::new(); for item in ");
                self.emit_expr(&args[0]);
                self.write(".iter() { *counts.entry(item.clone()).or_insert(0) += 1; } ");
                self.write("let pairs: Vec<String> = counts.iter().map(|(k, v)| format!(\"\\\"{}\\\":{}\", k, v)).collect(); ");
                self.write("format!(\"{{{}}}\", pairs.join(\",\")) }");
            }
            "counter_get" => {
                self.write(
                    "{ let data: std::collections::HashMap<String, i64> = serde_json::from_str(",
                );
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").unwrap_or_default(); let __key = ");
                self.emit_expr(&args[1]);
                self.write("; *data.get(__key.as_str()).unwrap_or(&0) }");
            }
            "counter_most_common" => {
                self.write(
                    "{ let data: std::collections::HashMap<String, i64> = serde_json::from_str(",
                );
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").unwrap_or_default(); let mut pairs: Vec<(String, i64)> = data.into_iter().collect(); ");
                self.write("pairs.sort_by(|a, b| b.1.cmp(&a.1)); pairs.truncate(");
                self.emit_expr(&args[1]);
                self.write(" as usize); ");
                self.write("let items: Vec<String> = pairs.iter().map(|(k, v)| format!(\"[\\\"{}\\\",{}]\", k, v)).collect(); ");
                self.write("format!(\"[{}]\", items.join(\",\")) }");
            }
            "counter_total" => {
                self.write(
                    "{ let data: std::collections::HashMap<String, i64> = serde_json::from_str(",
                );
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").unwrap_or_default(); data.values().sum::<i64>() }");
            }
            "counter_values" => {
                self.write(
                    "{ let data: std::collections::HashMap<String, i64> = serde_json::from_str(",
                );
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").unwrap_or_default(); data.values().cloned().collect::<Vec<i64>>() }");
            }
            "counter_keys" => {
                self.write(
                    "{ let data: std::collections::HashMap<String, i64> = serde_json::from_str(",
                );
                self.emit_expr_as_str_ref(&args[0]);
                self.write(
                    ").unwrap_or_default(); data.keys().cloned().collect::<Vec<String>>() }",
                );
            }
            "counter_items" => {
                self.write(
                    "{ let data: std::collections::HashMap<String, i64> = serde_json::from_str(",
                );
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").unwrap_or_default(); let mut pairs: Vec<(String, i64)> = data.into_iter().collect(); ");
                self.write("pairs.sort_by(|a, b| a.0.cmp(&b.0)); ");
                self.write("let items: Vec<String> = pairs.iter().map(|(k, v)| format!(\"[\\\"{}\\\",{}]\", k, v)).collect(); ");
                self.write("format!(\"[{}]\", items.join(\",\")) }");
            }
            "counter_increment" => {
                self.write("{ let mut data: std::collections::HashMap<String, i64> = serde_json::from_str(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").unwrap_or_default(); *data.entry(");
                self.emit_expr(&args[1]);
                self.write(".to_string()).or_insert(0) += 1; ");
                self.write("let pairs: Vec<String> = data.iter().map(|(k, v)| format!(\"\\\"{}\\\":{}\", k, v)).collect(); ");
                self.write("format!(\"{{{}}}\", pairs.join(\",\")) }");
            }
            // sifr.collections — DefaultDict
            "defaultdict_new" => {
                self.write("format!(\"{{\\\"__default__\\\":{}}}\", ");
                self.emit_expr(&args[0]);
                self.write(")");
            }
            "defaultdict_get" => {
                self.write(
                    "{ let data: std::collections::HashMap<String, i64> = serde_json::from_str(",
                );
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").unwrap_or_default(); let def = data.get(\"__default__\").cloned().unwrap_or(0); ");
                self.write("*data.get(");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").unwrap_or(&def) }");
            }
            "defaultdict_set" => {
                self.write("{ let mut data: std::collections::HashMap<String, serde_json::Value> = serde_json::from_str(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").unwrap_or_default(); data.insert(");
                self.emit_expr(&args[1]);
                self.write(".to_string(), serde_json::json!(");
                self.emit_expr(&args[2]);
                self.write(")); serde_json::to_string(&data).unwrap_or_default() }");
            }
            // sifr.bytes
            "encode_utf8" => {
                self.emit_expr_as_bytes(&args[0]);
                self.write(".iter().map(|b| *b as i64).collect::<Vec<i64>>()");
            }
            "decode_utf8" => {
                self.write("(|| -> Result<String, ParseError> { let __vals = ");
                self.emit_expr(&args[0]);
                self.write("; let mut __bytes: Vec<u8> = Vec::with_capacity(__vals.len()); for (__idx, __b) in __vals.iter().enumerate() { if *__b < 0 || *__b > 255 { return Err(ParseError { message: format!(\"byte out of range at index {}: {}\", __idx, *__b) }); } __bytes.push(*__b as u8); } String::from_utf8(__bytes).map_err(|e| ParseError { message: e.to_string() }) })()");
            }
            "bytes_to_hex" => {
                self.write("(|| -> Result<String, ParseError> { let __vals = ");
                self.emit_expr(&args[0]);
                self.write("; let mut __out = String::new(); for (__idx, __b) in __vals.iter().enumerate() { if *__b < 0 || *__b > 255 { return Err(ParseError { message: format!(\"byte out of range at index {}: {}\", __idx, *__b) }); } __out.push_str(&format!(\"{:02x}\", *__b as u8)); } Ok(__out) })()");
            }
            "bytes_from_hex" => {
                self.write("(|| -> Result<Vec<i64>, ParseError> { let s = ");
                self.emit_expr(&args[0]);
                self.write("; let mut cleaned = String::new(); for ch in s.chars() { if ch.is_ascii_whitespace() { continue; } if !ch.is_ascii_hexdigit() { return Err(ParseError { message: format!(\"invalid hex character: {}\", ch) }); } cleaned.push(ch); } if cleaned.len() % 2 != 0 { return Err(ParseError { message: \"fromhex() arg must contain an even number of hexadecimal digits\".to_string() }); } let mut result = Vec::new(); for pair in cleaned.as_bytes().chunks(2) { let pair_str = std::str::from_utf8(pair).map_err(|e| ParseError { message: e.to_string() })?; result.push(i64::from_str_radix(pair_str, 16).map_err(|e| ParseError { message: e.to_string() })?); } Ok(result) })()");
            }
            // sifr.time
            "time_now" => {
                self.write("std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs_f64()");
            }
            "sleep" => {
                self.write("std::thread::sleep(std::time::Duration::from_secs_f64(");
                self.emit_expr(&args[0]);
                self.write("))");
            }
            "time_format" => {
                self.write("{ let secs = ");
                self.emit_expr(&args[0]);
                self.write(" as i64; let dt = chrono::DateTime::from_timestamp(secs, 0).unwrap_or_default(); dt.format(");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").to_string() }");
            }
            "perf_counter" | "monotonic" => {
                self.write("{ fn __monotonic() -> f64 { static __START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new(); let s = __START.get_or_init(std::time::Instant::now); s.elapsed().as_secs_f64() } __monotonic() }");
            }
            // sifr.random
            "random_int" => {
                self.write("{ use rand::Rng; rand::thread_rng().gen_range(");
                self.emit_expr(&args[0]);
                self.write("..=");
                self.emit_expr(&args[1]);
                self.write(") }");
            }
            "random_float" => {
                self.write("{ use rand::Rng; rand::thread_rng().gen::<f64>() }");
            }
            "random_choice" => {
                self.write("{ use rand::Rng; let items = ");
                self.emit_expr(&args[0]);
                self.write("; items[rand::thread_rng().gen_range(0..items.len())].clone() }");
            }
            "random_uniform" => {
                self.write("{ use rand::Rng; rand::thread_rng().gen_range(");
                self.emit_expr(&args[0]);
                self.write("..=");
                self.emit_expr(&args[1]);
                self.write(") }");
            }
            "random_shuffle" => {
                self.write("{ use rand::seq::SliceRandom; let mut __v = ");
                self.emit_expr(&args[0]);
                self.write(".clone(); __v.shuffle(&mut rand::thread_rng()); __v }");
            }
            "random_sample" => {
                self.write("{ use rand::seq::SliceRandom; let __items = &");
                self.emit_expr(&args[0]);
                self.write("; let __k = ");
                self.emit_expr(&args[1]);
                self.write(" as usize; if __k > __items.len() { Err(ValueError { message: format!(\"sample larger than population: {} > {}\", __k, __items.len()) }) } else { Ok(__items.choose_multiple(&mut rand::thread_rng(), __k).cloned().collect::<Vec<_>>()) } }");
            }
            "random_randrange" => {
                self.write("{ let __start = ");
                self.emit_expr(&args[0]);
                self.write("; let __stop = ");
                self.emit_expr(&args[1]);
                self.write("; let __step = ");
                self.emit_expr(&args[2]);
                self.write("; if __step == 0 { Err(ValueError { message: \"randrange: step must not be zero\".to_string() }) } else if __start >= __stop && __step > 0 { Err(ValueError { message: \"randrange: empty range\".to_string() }) } else { use rand::Rng; let __n = ((__stop - __start + __step - 1) / __step).abs(); Ok(__start + rand::thread_rng().gen_range(0..__n) * __step) } }");
            }
            "random_gauss" => {
                self.write("{ use rand_distr::{Normal, Distribution}; let __mu = ");
                self.emit_expr(&args[0]);
                self.write("; let __sigma = ");
                self.emit_expr(&args[1]);
                self.write("; Normal::new(__mu, __sigma).map(|d| d.sample(&mut rand::thread_rng())).unwrap_or(__mu) }");
            }
            // sifr.re
            "re_match" => {
                self.write("regex::Regex::new(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|re| re.is_match(");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(
                    ")).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })",
                );
            }
            "re_find" => {
                self.write("regex::Regex::new(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|re| re.find(");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").map(|m| m.as_str().to_string())).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })");
            }
            "re_replace" => {
                self.write("regex::Regex::new(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|re| re.replace_all(");
                self.emit_expr_as_str_ref(&args[2]);
                self.write(", ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").to_string()).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })");
            }
            "re_findall" => {
                self.write("regex::Regex::new(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|re| re.find_iter(");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").map(|m| m.as_str().to_string()).collect::<Vec<String>>()).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })");
            }
            "re_split" => {
                self.write("regex::Regex::new(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|re| re.split(");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").map(|s| s.to_string()).collect::<Vec<String>>()).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })");
            }
            "re_find_start" => {
                self.write("regex::Regex::new(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|re| re.find(");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").map_or(-1_i64, |m| m.start() as i64)).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })");
            }
            "re_find_end" => {
                self.write("regex::Regex::new(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|re| re.find(");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").map_or(-1_i64, |m| m.end() as i64)).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })");
            }
            // re flags variants
            // Signatures:
            //   re_match_flags(pattern, text, flags)
            //   re_find_flags(pattern, text, flags)
            //   re_replace_flags(pattern, replacement, text, flags)
            //   re_findall_flags(pattern, text, flags)
            //   re_split_flags(pattern, text, flags)
            "re_match_flags" | "re_find_flags" | "re_findall_flags" | "re_split_flags" => {
                let flags_idx = 2usize;
                let text_idx = 1usize;
                self.write("(|| -> Result<");
                match func {
                    "re_match_flags" => self.write("bool"),
                    "re_find_flags" => self.write("Option<String>"),
                    _ => self.write("Vec<String>"),
                }
                self.write(", RegexError> { let __flags_val = ");
                self.emit_expr(&args[flags_idx]);
                self.write("; let mut __flag_str = String::new(); if __flags_val & 2 != 0 { __flag_str.push_str(\"(?i)\"); } if __flags_val & 8 != 0 { __flag_str.push_str(\"(?m)\"); } if __flags_val & 16 != 0 { __flag_str.push_str(\"(?s)\"); } if __flags_val & 64 != 0 { __flag_str.push_str(\"(?x)\"); } let __pat = __flag_str + ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __re = regex::Regex::new(&__pat).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })?; ");
                match func {
                    "re_match_flags" => {
                        self.write("Ok(__re.is_match(");
                        self.emit_expr_as_str_ref(&args[text_idx]);
                        self.write("))");
                    }
                    "re_find_flags" => {
                        self.write("Ok(__re.find(");
                        self.emit_expr_as_str_ref(&args[text_idx]);
                        self.write(").map(|m| m.as_str().to_string()))");
                    }
                    "re_findall_flags" => {
                        self.write("Ok(__re.find_iter(");
                        self.emit_expr_as_str_ref(&args[text_idx]);
                        self.write(").map(|m| m.as_str().to_string()).collect())");
                    }
                    "re_split_flags" => {
                        self.write("Ok(__re.split(");
                        self.emit_expr_as_str_ref(&args[text_idx]);
                        self.write(").map(|s| s.to_string()).collect())");
                    }
                    _ => {}
                }
                self.write(" })()");
            }
            "re_replace_flags" => {
                // re_replace_flags(pattern, replacement, text, flags)
                self.write("(|| -> Result<String, RegexError> { let __flags_val = ");
                self.emit_expr(&args[3]);
                self.write("; let mut __flag_str = String::new(); if __flags_val & 2 != 0 { __flag_str.push_str(\"(?i)\"); } if __flags_val & 8 != 0 { __flag_str.push_str(\"(?m)\"); } if __flags_val & 16 != 0 { __flag_str.push_str(\"(?s)\"); } if __flags_val & 64 != 0 { __flag_str.push_str(\"(?x)\"); } let __pat = __flag_str + ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __re = regex::Regex::new(&__pat).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })?; Ok(__re.replace_all(");
                self.emit_expr_as_str_ref(&args[2]);
                self.write(", ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").to_string()) })()");
            }
            // sifr.hash
            "sha256" => {
                self.write("{ use sha2::Digest; format!(\"{:x}\", sha2::Sha256::digest(");
                self.emit_expr_as_bytes(&args[0]);
                self.write(")) }");
            }
            "md5" => {
                self.write("format!(\"{:x}\", md5::compute(");
                self.emit_expr_as_bytes(&args[0]);
                self.write("))");
            }
            // sifr.encoding
            "base64_encode" => {
                self.write(
                    "{ use base64::Engine; base64::engine::general_purpose::STANDARD.encode(",
                );
                self.emit_expr_as_bytes(&args[0]);
                self.write(") }");
            }
            "base64_decode" => {
                self.write("(|| -> Result<String, ParseError> { use base64::Engine; let bytes = base64::engine::general_purpose::STANDARD.decode(");
                self.emit_expr_as_bytes(&args[0]);
                self.write(").map_err(|e| ParseError { message: e.to_string() })?; String::from_utf8(bytes).map_err(|e| ParseError { message: e.to_string() }) })()");
            }
            "base64_encode_opts" => {
                self.write("(|| -> Result<String, ParseError> { use base64::Engine; let __s = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __alt = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; let __wrap = ");
                self.emit_expr(&args[2]);
                self.write("; if __wrap < 0 { return Err(ParseError { message: \"wrapcol must be >= 0\".to_string() }); } let mut __encoded = base64::engine::general_purpose::STANDARD.encode(__s.as_bytes()); if !__alt.is_empty() { if __alt.chars().count() != 2 { return Err(ParseError { message: format!(\"invalid altchars: {}\", __alt) }); } let mut __it = __alt.chars(); let __a = __it.next().unwrap_or('+'); let __b = __it.next().unwrap_or('/'); __encoded = __encoded.chars().map(|c| if c == '+' { __a } else if c == '/' { __b } else { c }).collect::<String>(); } if __wrap == 0 { return Ok(__encoded); } let __w = __wrap as usize; let mut __wrapped = String::new(); for (i, ch) in __encoded.chars().enumerate() { if i > 0 && i % __w == 0 { __wrapped.push('\\n'); } __wrapped.push(ch); } Ok(__wrapped) })()");
            }
            "base64_decode_opts" => {
                self.write("(|| -> Result<String, ParseError> { use base64::Engine; let __s = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __alt = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; let __validate = ");
                self.emit_expr(&args[2]);
                self.write("; let __ignore = ");
                self.emit_expr_as_str_ref(&args[3]);
                self.write("; let mut __has_alt = false; let mut __alt_a = '+'; let mut __alt_b = '/'; if !__alt.is_empty() { if __alt.chars().count() != 2 { return Err(ParseError { message: format!(\"invalid altchars: {}\", __alt) }); } let mut __it = __alt.chars(); __alt_a = __it.next().unwrap_or('+'); __alt_b = __it.next().unwrap_or('/'); __has_alt = true; } let mut __ignore_set = std::collections::HashSet::<char>::new(); for ch in __ignore.chars() { __ignore_set.insert(ch); } let mut __normalized = String::new(); for ch in __s.chars() { if __ignore_set.contains(&ch) { continue; } let mut mapped = ch; if __has_alt { if ch == __alt_a { mapped = '+'; } else if ch == __alt_b { mapped = '/'; } } let is_base64 = (mapped >= 'A' && mapped <= 'Z') || (mapped >= 'a' && mapped <= 'z') || (mapped >= '0' && mapped <= '9') || mapped == '+' || mapped == '/' || mapped == '='; if is_base64 { __normalized.push(mapped); } else if __validate { return Err(ParseError { message: format!(\"invalid base64 character: {}\", ch) }); } } let __bytes = base64::engine::general_purpose::STANDARD.decode(__normalized.as_bytes()).map_err(|e| ParseError { message: e.to_string() })?; String::from_utf8(__bytes).map_err(|e| ParseError { message: e.to_string() }) })()");
            }
            "sha1" => {
                self.write("{ use sha1::Digest; format!(\"{:x}\", sha1::Sha1::digest(");
                self.emit_expr_as_bytes(&args[0]);
                self.write(")) }");
            }
            "sha512" => {
                self.write("{ use sha2::Digest; format!(\"{:x}\", sha2::Sha512::digest(");
                self.emit_expr_as_bytes(&args[0]);
                self.write(")) }");
            }
            "urlsafe_b64encode" => {
                self.write(
                    "{ use base64::Engine; base64::engine::general_purpose::URL_SAFE.encode(",
                );
                self.emit_expr_as_bytes(&args[0]);
                self.write(") }");
            }
            "urlsafe_b64decode" => {
                self.write("(|| -> Result<String, ParseError> { use base64::Engine; let bytes = base64::engine::general_purpose::URL_SAFE.decode(");
                self.emit_expr_as_bytes(&args[0]);
                self.write(").map_err(|e| ParseError { message: e.to_string() })?; String::from_utf8(bytes).map_err(|e| ParseError { message: e.to_string() }) })()");
            }
            // sifr.uuid
            "uuid4" => {
                self.write("{ use rand::Rng; let mut rng = rand::thread_rng(); let bytes: [u8; 16] = rng.gen(); format!(\"{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}\", u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]), u16::from_be_bytes([bytes[4], bytes[5]]), u16::from_be_bytes([bytes[6], bytes[7]]) & 0x0fff, (u16::from_be_bytes([bytes[8], bytes[9]]) & 0x3fff) | 0x8000, u64::from_be_bytes([0, 0, bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]])) }");
            }
            // sifr.platform
            "platform_system" => {
                self.write("std::env::consts::OS.to_string()");
            }
            "platform_arch" => {
                self.write("std::env::consts::ARCH.to_string()");
            }
            "platform_node" => {
                self.write("std::process::Command::new(\"hostname\").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default()");
            }
            // sifr.toml
            "toml_parse" => {
                self.write("{ let __toml_str = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; __toml_str.parse::<toml::Value>().map(|v| format!(\"{}\", v)).map_err(|e| TOMLDecodeError { message: e.to_string(), line: 0, column: 0 }) }");
            }
            // sifr.datetime
            "datetime_now" => {
                self.write("chrono::Local::now().format(\"%Y-%m-%dT%H:%M:%S\").to_string()");
            }
            "datetime_now_struct" => {
                self.write("{ use chrono::{Datelike, Timelike}; let __dt = chrono::Local::now(); vec![__dt.year() as i64, __dt.month() as i64, __dt.day() as i64, __dt.hour() as i64, __dt.minute() as i64, __dt.second() as i64] }");
            }
            "time_strptime" => {
                self.write("(|| -> Result<Vec<i64>, ValueError> { let __s = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __fmt = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; chrono::NaiveDateTime::parse_from_str(__s, __fmt).map(|dt| { use chrono::Datelike; use chrono::Timelike; vec![dt.year() as i64, dt.month() as i64, dt.day() as i64, dt.hour() as i64, dt.minute() as i64, dt.second() as i64, dt.weekday().num_days_from_monday() as i64, dt.ordinal() as i64] }).map_err(|e| ValueError { message: e.to_string() }) })()");
            }
            "time_gmtime" => {
                self.write("{ use chrono::{Datelike, Timelike, Utc}; let __dt = Utc::now().naive_utc(); vec![__dt.year() as i64, __dt.month() as i64, __dt.day() as i64, __dt.hour() as i64, __dt.minute() as i64, __dt.second() as i64, __dt.weekday().num_days_from_monday() as i64, __dt.ordinal() as i64] }");
            }
            "time_localtime" => {
                self.write("{ use chrono::{Datelike, Timelike, Local}; let __dt = Local::now().naive_local(); vec![__dt.year() as i64, __dt.month() as i64, __dt.day() as i64, __dt.hour() as i64, __dt.minute() as i64, __dt.second() as i64, __dt.weekday().num_days_from_monday() as i64, __dt.ordinal() as i64] }");
            }
            "datetime_format" => {
                self.write("{ let __dt_str = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __fmt = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; __dt_str.to_string() }");
            }
            "datetime_from_timestamp" => {
                self.write("{ let __ts = ");
                self.emit_expr(&args[0]);
                self.write(" as i64; chrono::DateTime::from_timestamp(__ts, 0).map(|dt| dt.format(\"%Y-%m-%dT%H:%M:%S\").to_string()).ok_or_else(|| ValueError { message: \"invalid timestamp\".to_string() }) }");
            }
            // sifr.math new intrinsics
            "erf" => {
                self.write("{ let __x: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; let __t = 1.0 / (1.0 + 0.3275911 * __x.abs()); let __poly = __t * (0.254829592 + __t * (-0.284496736 + __t * (1.421413741 + __t * (-1.453152027 + __t * 1.061405429)))); let __r = 1.0 - __poly * (-__x * __x).exp(); if __x >= 0.0 { __r } else { -__r } }");
            }
            "erfc" => {
                self.write("{ let __x: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; let __t = 1.0 / (1.0 + 0.3275911 * __x.abs()); let __poly = __t * (0.254829592 + __t * (-0.284496736 + __t * (1.421413741 + __t * (-1.453152027 + __t * 1.061405429)))); let __r = __poly * (-__x * __x).exp(); if __x >= 0.0 { __r } else { 2.0 - __r } }");
            }
            "gamma" => {
                self.write("{ let __x: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; if __x <= 0.0 && __x == __x.floor() { f64::INFINITY } else { let __g = 7usize; let __c = [0.99999999999980993f64, 676.5203681218851, -1259.1392167224028, 771.32342877765313, -176.61502916214059, 12.507343278686905, -0.13857109526572012, 9.9843695780195716e-6, 1.5056327351493116e-7]; let __z = if __x < 0.5 { let __y = std::f64::consts::PI / ((__x * std::f64::consts::PI).sin() * { let __xn = 1.0 - __x; let mut __s = __c[0]; for __i in 1..=__g+1 { __s += __c[__i] / (__xn + __i as f64 - 1.0); } let __t2 = __xn + __g as f64 - 0.5; (2.0 * std::f64::consts::PI).sqrt() * __t2.powf(__xn - 0.5) * (-__t2).exp() * __s }); __y } else { let __xm = __x - 1.0; let mut __s = __c[0]; for __i in 1..=__g+1 { __s += __c[__i] / (__xm + __i as f64); } let __t2 = __xm + __g as f64 + 0.5; (2.0 * std::f64::consts::PI).sqrt() * __t2.powf(__xm + 0.5) * (-__t2).exp() * __s }; __z } }");
            }
            "lgamma" => {
                self.write("{ let __x: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; if __x <= 0.0 && __x == __x.floor() { f64::INFINITY } else { let __g = 7usize; let __c = [0.99999999999980993f64, 676.5203681218851, -1259.1392167224028, 771.32342877765313, -176.61502916214059, 12.507343278686905, -0.13857109526572012, 9.9843695780195716e-6, 1.5056327351493116e-7]; let __xm = if __x < 0.5 { 1.0 - __x } else { __x - 1.0 }; let mut __s = __c[0]; for __i in 1..=__g+1 { __s += __c[__i] / (__xm + __i as f64); } let __t2 = __xm + __g as f64 + 0.5; let __r = (2.0 * std::f64::consts::PI).sqrt().ln() + (__xm + 0.5) * __t2.ln() - __t2 + __s.ln(); if __x < 0.5 { (std::f64::consts::PI / ((__x * std::f64::consts::PI).sin() * __r.exp())).abs().ln() } else { __r } } }");
            }
            "frexp" => {
                self.write("{ let __x: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; if __x == 0.0 { vec![__x, 0.0] } else if !__x.is_finite() { vec![__x, 0.0] } else { let __bits = __x.to_bits(); let __sign = __bits & 0x8000000000000000; let __exp = ((__bits >> 52) & 0x7ff) as i32; let __frac = __bits & 0x000fffffffffffff; if __exp == 0 { let __scaled = __x * (2.0f64).powi(54); let __sbits = __scaled.to_bits(); let __sexp = ((__sbits >> 52) & 0x7ff) as i32; let __sfrac = __sbits & 0x000fffffffffffff; let __mant = f64::from_bits(__sign | (0x3feu64 << 52) | __sfrac); let __e = __sexp - 1022 - 54; vec![__mant, __e as f64] } else { let __mant = f64::from_bits(__sign | (0x3feu64 << 52) | __frac); let __e = __exp - 1022; vec![__mant, __e as f64] } } }");
            }
            "ldexp" => {
                self.write("{ let __m: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; let __e: i64 = ");
                self.emit_expr(&args[1]);
                self.write("; __m * (2.0f64).powi(__e as i32) }");
            }
            "modf" => {
                self.write("{ let __x: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; if __x.is_nan() { vec![f64::NAN, f64::NAN] } else if __x.is_infinite() { vec![0.0f64.copysign(__x), __x] } else { let __int = __x.trunc(); let mut __frac = __x - __int; if __frac == 0.0 { __frac = 0.0f64.copysign(__x); } vec![__frac, __int] } }");
            }
            "nextafter" => {
                self.write("{ let __x: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; let __y: f64 = ");
                self.emit_expr(&args[1]);
                self.write("; if __x.is_nan() || __y.is_nan() { f64::NAN } else if __x == __y { __y } else if __x == 0.0 { let __sign = if __y.is_sign_negative() { 1u64 << 63 } else { 0u64 }; f64::from_bits(__sign | 1u64) } else { let mut __bits = __x.to_bits(); if (__x < __y) == (__x > 0.0) { __bits += 1; } else { __bits -= 1; } f64::from_bits(__bits) } }");
            }
            "ulp" => {
                self.write("{ let __x: f64 = ");
                self.emit_expr(&args[0]);
                self.write("; if __x.is_nan() { f64::NAN } else if __x.is_infinite() { f64::INFINITY } else { let __a = __x.abs(); if __a == 0.0 { f64::from_bits(1u64) } else if __a == f64::MAX { __a - f64::from_bits(__a.to_bits() - 1) } else { f64::from_bits(__a.to_bits() + 1) - __a } } }");
            }
            // sifr.pathlib new intrinsics
            "touch" => {
                self.write("std::fs::OpenOptions::new().create(true).write(true).open(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|_| ()).map_err(__io_err)");
            }
            "resolve_path" => {
                self.write("std::fs::canonicalize(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|p| p.to_string_lossy().to_string()).map_err(__io_err)");
            }
            "iterdir" => {
                self.write(
                    "(|| -> Result<Vec<String>, IOError> { let __entries = std::fs::read_dir(",
                );
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map_err(__io_err)?; Ok(__entries.filter_map(|e| e.ok().map(|e| e.path().to_string_lossy().to_string())).collect()) })()");
            }
            // sifr.os new intrinsics
            "chdir" => {
                self.write("std::env::set_current_dir(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map_err(__io_err)");
            }
            "getpid" => {
                self.write("std::process::id() as i64");
            }
            "cpu_count" => {
                self.write("{ let __n = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1); __n as i64 }");
            }
            "stat_size" => {
                self.write("std::fs::metadata(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map(|m| m.len() as i64).map_err(__io_err)");
            }
            "which" => {
                self.write("{ let __name = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; std::env::var(\"PATH\").ok().and_then(|__path| __path.split(':').map(|d| std::path::Path::new(d).join(__name)).find(|p| p.is_file()).map(|p| p.to_string_lossy().to_string())) }");
            }
            "disk_usage" => {
                self.write("{ let __path = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __stat = std::fs::metadata(__path); match __stat { Ok(_) => { let __out = std::process::Command::new(\"df\").args([\"-k\", __path]).output(); match __out { Ok(__o) => { let __s = String::from_utf8_lossy(&__o.stdout); let __lines: Vec<&str> = __s.lines().collect(); if __lines.len() >= 2 { let __parts: Vec<&str> = __lines[1].split_whitespace().collect(); if __parts.len() >= 4 { let __total = __parts[1].parse::<i64>().unwrap_or(0) * 1024; let __used = __parts[2].parse::<i64>().unwrap_or(0) * 1024; let __free = __parts[3].parse::<i64>().unwrap_or(0) * 1024; vec![__total, __used, __free] } else { vec![0i64, 0, 0] } } else { vec![0i64, 0, 0] } }, Err(_) => vec![0i64, 0, 0] } }, Err(_) => vec![0i64, 0, 0] } }");
            }
            // open() built-in — wraps open_file and constructs FileHandle (raises IOError on failure)
            "builtin_open" => {
                self.runtime_needs.needs_file_handles = true;
                self.used_stdlib_modules.insert("io".to_string());
                self.write("{ use std::io::{BufReader, BufWriter}; let __path = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __mode = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; let __handle_id: i64 = { use std::sync::atomic::{AtomicI64, Ordering}; static __NEXT_FH_ID: AtomicI64 = AtomicI64::new(1); __NEXT_FH_ID.fetch_add(1, Ordering::SeqCst) }; match __mode { \"r\" | \"rt\" => { let __f = std::fs::File::open(__path).map_err(__io_err)?; let __reader = BufReader::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::TextRead(__reader)); FileHandle { _handle: __handle_id, _mode: __mode.to_string() } }, \"w\" | \"wt\" => { let __f = std::fs::File::create(__path).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::TextWrite(__writer)); FileHandle { _handle: __handle_id, _mode: __mode.to_string() } }, \"a\" | \"at\" => { let __f = std::fs::OpenOptions::new().append(true).create(true).open(__path).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::TextWrite(__writer)); FileHandle { _handle: __handle_id, _mode: __mode.to_string() } }, \"rb\" => { let __f = std::fs::File::open(__path).map_err(__io_err)?; let __reader = BufReader::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::BinaryRead(__reader)); FileHandle { _handle: __handle_id, _mode: __mode.to_string() } }, \"wb\" => { let __f = std::fs::File::create(__path).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::BinaryWrite(__writer)); FileHandle { _handle: __handle_id, _mode: __mode.to_string() } }, \"ab\" => { let __f = std::fs::OpenOptions::new().append(true).create(true).open(__path).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::BinaryWrite(__writer)); FileHandle { _handle: __handle_id, _mode: __mode.to_string() } }, _ => return Err(IOError { message: format!(\"invalid mode: {}\", __mode), kind: \"Other\".to_string() }) } }");
            }
            // open() built-in file handle intrinsics
            "open_file" => {
                self.runtime_needs.needs_file_handles = true;
                self.write("(|| -> Result<i64, IOError> { use std::io::{BufReader, BufWriter}; let __path = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __mode = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; let __handle_id: i64 = { use std::sync::atomic::{AtomicI64, Ordering}; static __NEXT_ID: AtomicI64 = AtomicI64::new(1); __NEXT_ID.fetch_add(1, Ordering::SeqCst) }; let __mode_s: &str = &__mode; let __path_s: &str = &__path; match __mode_s { \"r\" | \"rt\" => { let __f = std::fs::File::open(__path_s).map_err(__io_err)?; let __reader = BufReader::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::TextRead(__reader)); Ok(__handle_id) }, \"w\" | \"wt\" => { let __f = std::fs::File::create(__path_s).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::TextWrite(__writer)); Ok(__handle_id) }, \"a\" | \"at\" => { let __f = std::fs::OpenOptions::new().append(true).create(true).open(__path_s).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::TextWrite(__writer)); Ok(__handle_id) }, \"rb\" => { let __f = std::fs::File::open(__path_s).map_err(__io_err)?; let __reader = BufReader::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::BinaryRead(__reader)); Ok(__handle_id) }, \"wb\" => { let __f = std::fs::File::create(__path_s).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::BinaryWrite(__writer)); Ok(__handle_id) }, \"ab\" => { let __f = std::fs::OpenOptions::new().append(true).create(true).open(__path_s).map_err(__io_err)?; let __writer = BufWriter::new(__f); __SIFR_FILE_HANDLES.lock().unwrap().insert(__handle_id, SifrFileHandle::BinaryWrite(__writer)); Ok(__handle_id) }, _ => Err(IOError { message: format!(\"invalid mode: {}\", __mode), kind: \"Other\".to_string() }) } })()");
            }
            "file_read" => {
                self.write("(|| -> Result<String, IOError> { use std::io::Read; let __hid = ");
                self.emit_expr(&args[0]);
                self.write("; let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) { Some(SifrFileHandle::TextRead(ref mut __r)) => { let mut __s = String::new(); __r.read_to_string(&mut __s).map_err(__io_err)?; Ok(__s) }, _ => Err(IOError { message: \"file not open for reading\".to_string(), kind: \"Other\".to_string() }) } })()");
            }
            "file_write" => {
                self.write("(|| -> Result<(), IOError> { use std::io::Write; let __hid = ");
                self.emit_expr(&args[0]);
                self.write("; let __data = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) { Some(SifrFileHandle::TextWrite(ref mut __w)) => { __w.write_all(__data.as_bytes()).map_err(__io_err)?; Ok(()) }, _ => Err(IOError { message: \"file not open for writing\".to_string(), kind: \"Other\".to_string() }) } })()");
            }
            "file_readline" => {
                self.write(
                    "(|| -> Result<Option<String>, IOError> { use std::io::BufRead; let __hid = ",
                );
                self.emit_expr(&args[0]);
                self.write("; let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) { Some(SifrFileHandle::TextRead(ref mut __r)) => { let mut __line = String::new(); let __n = __r.read_line(&mut __line).map_err(__io_err)?; if __n == 0 { Ok(None) } else { if __line.ends_with('\\n') { __line.pop(); if __line.ends_with('\\r') { __line.pop(); } } Ok(Some(__line)) } }, _ => Err(IOError { message: \"file not open for reading\".to_string(), kind: \"Other\".to_string() }) } })()");
            }
            "file_readlines" => {
                self.write(
                    "(|| -> Result<Vec<String>, IOError> { use std::io::BufRead; let __hid = ",
                );
                self.emit_expr(&args[0]);
                self.write("; let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) { Some(SifrFileHandle::TextRead(ref mut __r)) => { let mut __lines: Vec<String> = Vec::new(); let mut __line = String::new(); loop { __line.clear(); let __n = __r.read_line(&mut __line).map_err(__io_err)?; if __n == 0 { break; } let mut __l = __line.clone(); if __l.ends_with('\\n') { __l.pop(); if __l.ends_with('\\r') { __l.pop(); } } __lines.push(__l); } Ok(__lines) }, _ => Err(IOError { message: \"file not open for reading\".to_string(), kind: \"Other\".to_string() }) } })()");
            }
            "file_close" => {
                self.write("{ let __hid = ");
                self.emit_expr(&args[0]);
                self.write("; __SIFR_FILE_HANDLES.lock().unwrap().remove(&__hid); }");
            }
            "file_read_bytes" => {
                self.write("(|| -> Result<Vec<i64>, IOError> { use std::io::Read; let __hid = ");
                self.emit_expr(&args[0]);
                self.write("; let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) { Some(SifrFileHandle::BinaryRead(ref mut __r)) => { let mut __buf = Vec::new(); __r.read_to_end(&mut __buf).map_err(__io_err)?; Ok(__buf.iter().map(|&b| b as i64).collect()) }, _ => Err(IOError { message: \"file not open for binary reading\".to_string(), kind: \"Other\".to_string() }) } })()");
            }
            "file_write_bytes" => {
                self.write("(|| -> Result<(), IOError> { use std::io::Write; let __hid = ");
                self.emit_expr(&args[0]);
                self.write("; let __data: Vec<u8> = ");
                self.emit_expr(&args[1]);
                self.write(".iter().map(|&b| b as u8).collect(); let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) { Some(SifrFileHandle::BinaryWrite(ref mut __w)) => { __w.write_all(&__data).map_err(__io_err)?; Ok(()) }, _ => Err(IOError { message: \"file not open for binary writing\".to_string(), kind: \"Other\".to_string() }) } })()");
            }
            // Path.glob / Path.rglob
            "glob_pattern" => {
                self.write("(|| -> Result<Vec<String>, IOError> { let __dir = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __pat = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; let __full_pat = if __dir.is_empty() { __pat.to_string() } else { format!(\"{}/{}\", __dir, __pat) }; let __entries = std::fs::read_dir(__dir).map_err(__io_err)?; let mut __results: Vec<String> = Vec::new(); fn __matches_glob(name: &str, pattern: &str) -> bool { let __parts: Vec<&str> = pattern.split('*').collect(); if __parts.len() == 1 { return name == pattern; } if !name.starts_with(__parts[0]) { return false; } let mut __pos = __parts[0].len(); for __i in 1..__parts.len() { if __parts[__i].is_empty() { __pos = name.len(); continue; } match name[__pos..].find(__parts[__i]) { Some(__idx) => __pos += __idx + __parts[__i].len(), None => return false, } } true } for __entry in __entries { let __e = __entry.map_err(__io_err)?; let __name = __e.file_name().to_string_lossy().to_string(); if __matches_glob(&__name, __pat) { __results.push(__e.path().to_string_lossy().to_string()); } } __results.sort(); Ok(__results) })()");
            }
            "rglob_pattern" => {
                self.write("(|| -> Result<Vec<String>, IOError> { let __dir = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __pat = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; fn __rglob_walk(dir: &str, pattern: &str, results: &mut Vec<String>) -> Result<(), IOError> { fn __io_err_inner(e: std::io::Error) -> IOError { IOError { message: e.to_string(), kind: \"Other\".to_string() } } fn __matches_glob(name: &str, pat: &str) -> bool { let parts: Vec<&str> = pat.split('*').collect(); if parts.len() == 1 { return name == pat; } if !name.starts_with(parts[0]) { return false; } let mut pos = parts[0].len(); for i in 1..parts.len() { if parts[i].is_empty() { pos = name.len(); continue; } match name[pos..].find(parts[i]) { Some(idx) => pos += idx + parts[i].len(), None => return false, } } true } let entries = std::fs::read_dir(dir).map_err(__io_err_inner)?; for entry in entries { let e = entry.map_err(__io_err_inner)?; let path = e.path(); let name = e.file_name().to_string_lossy().to_string(); if path.is_dir() { __rglob_walk(&path.to_string_lossy(), pattern, results)?; } if __matches_glob(&name, pattern) { results.push(path.to_string_lossy().to_string()); } } Ok(()) } let mut __results: Vec<String> = Vec::new(); __rglob_walk(__dir, __pat, &mut __results).map_err(|e| e)?; __results.sort(); Ok(__results) })()");
            }
            // os constants
            "os_sep" => {
                self.write("std::path::MAIN_SEPARATOR.to_string()");
            }
            "os_linesep" => {
                #[cfg(target_os = "windows")]
                self.write("\"\\r\\n\".to_string()");
                #[cfg(not(target_os = "windows"))]
                self.write("\"\\n\".to_string()");
            }
            "os_name" => {
                self.write("{ if cfg!(target_os = \"windows\") { \"nt\".to_string() } else { \"posix\".to_string() } }");
            }
            // sifr.hashlib new intrinsics
            "sha224" => {
                self.write("{ use sha2::Digest; let mut __h = sha2::Sha224::new(); __h.update(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(".as_bytes()); format!(\"{:x}\", __h.finalize()) }");
            }
            "sha384" => {
                self.write("{ use sha2::Digest; let mut __h = sha2::Sha384::new(); __h.update(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(".as_bytes()); format!(\"{:x}\", __h.finalize()) }");
            }
            "blake2b" => {
                self.write("{ use blake2::{Blake2b512, Digest}; let mut __h = Blake2b512::new(); __h.update(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(".as_bytes()); format!(\"{:x}\", __h.finalize()) }");
            }
            "blake2s" => {
                self.write("{ use blake2::{Blake2s256, Digest}; let mut __h = Blake2s256::new(); __h.update(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(".as_bytes()); format!(\"{:x}\", __h.finalize()) }");
            }
            // sifr.base64 new intrinsics
            "b32encode" => {
                self.write(
                    "{ let __b32_alpha = b\"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567\"; let __data = ",
                );
                self.emit_expr_as_str_ref(&args[0]);
                self.write(".as_bytes(); let mut __out = String::new(); let mut __i = 0usize; while __i < __data.len() { let __b0 = __data[__i] as u64; let __b1 = if __i+1 < __data.len() { __data[__i+1] as u64 } else { 0 }; let __b2 = if __i+2 < __data.len() { __data[__i+2] as u64 } else { 0 }; let __b3 = if __i+3 < __data.len() { __data[__i+3] as u64 } else { 0 }; let __b4 = if __i+4 < __data.len() { __data[__i+4] as u64 } else { 0 }; let __buf = (__b0<<32)|(__b1<<24)|(__b2<<16)|(__b3<<8)|__b4; let __n = ((__data.len() - __i).min(5)) as u64; for __j in 0..8u64 { if __j < (__n*8+4)/5 { __out.push(__b32_alpha[((__buf >> (35 - __j*5)) & 0x1f) as usize] as char); } else { __out.push('='); } } __i += 5; } __out }");
            }
            "b32decode" => {
                self.write("(|| -> Result<String, ParseError> { let __b32_alpha = b\"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567\"; let __s = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __s = __s.trim_end_matches('='); let mut __bits = 0u64; let mut __bit_count = 0u32; let mut __out: Vec<u8> = Vec::new(); for __c in __s.chars() { let __val = __b32_alpha.iter().position(|&b| b as char == __c.to_ascii_uppercase()).ok_or_else(|| ParseError { message: format!(\"invalid base32 char: {}\", __c) })? as u64; __bits = (__bits << 5) | __val; __bit_count += 5; if __bit_count >= 8 { __bit_count -= 8; __out.push(((__bits >> __bit_count) & 0xff) as u8); } } String::from_utf8(__out).map_err(|e| ParseError { message: e.to_string() }) })()");
            }
            "b32hexencode" => {
                self.write(
                    "{ let __b32_alpha = b\"0123456789ABCDEFGHIJKLMNOPQRSTUV\"; let __data = ",
                );
                self.emit_expr_as_str_ref(&args[0]);
                self.write(".as_bytes(); let mut __out = String::new(); let mut __i = 0usize; while __i < __data.len() { let __b0 = __data[__i] as u64; let __b1 = if __i+1 < __data.len() { __data[__i+1] as u64 } else { 0 }; let __b2 = if __i+2 < __data.len() { __data[__i+2] as u64 } else { 0 }; let __b3 = if __i+3 < __data.len() { __data[__i+3] as u64 } else { 0 }; let __b4 = if __i+4 < __data.len() { __data[__i+4] as u64 } else { 0 }; let __buf = (__b0<<32)|(__b1<<24)|(__b2<<16)|(__b3<<8)|__b4; let __n = ((__data.len() - __i).min(5)) as u64; for __j in 0..8u64 { if __j < (__n*8+4)/5 { __out.push(__b32_alpha[((__buf >> (35 - __j*5)) & 0x1f) as usize] as char); } else { __out.push('='); } } __i += 5; } __out }");
            }
            "b32hexdecode" => {
                self.write("(|| -> Result<String, ParseError> { let __b32_alpha = b\"0123456789ABCDEFGHIJKLMNOPQRSTUV\"; let __s = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __s = __s.trim_end_matches('='); let mut __bits = 0u64; let mut __bit_count = 0u32; let mut __out: Vec<u8> = Vec::new(); for __c in __s.chars() { let __val = __b32_alpha.iter().position(|&b| b as char == __c.to_ascii_uppercase()).ok_or_else(|| ParseError { message: format!(\"invalid base32hex char: {}\", __c) })? as u64; __bits = (__bits << 5) | __val; __bit_count += 5; if __bit_count >= 8 { __bit_count -= 8; __out.push(((__bits >> __bit_count) & 0xff) as u8); } } String::from_utf8(__out).map_err(|e| ParseError { message: e.to_string() }) })()");
            }
            // sifr.platform new intrinsics
            "platform_release" => {
                self.write("{ std::process::Command::new(\"uname\").arg(\"-r\").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default() }");
            }
            "platform_version" => {
                self.write("{ std::process::Command::new(\"uname\").arg(\"-v\").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default() }");
            }
            "platform_processor" => {
                self.write("std::env::consts::ARCH.to_string()");
            }
            // sifr.time new intrinsics
            "strptime" => {
                self.write(
                    "(|| -> Result<String, ValueError> { use chrono::NaiveDateTime; let __s = ",
                );
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __fmt = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; NaiveDateTime::parse_from_str(__s, __fmt).map(|dt| dt.format(\"%Y-%m-%dT%H:%M:%S\").to_string()).map_err(|e| ValueError { message: e.to_string() }) })()");
            }
            "gmtime" => {
                self.write("{ use chrono::{DateTime, Utc}; let __ts = ");
                self.emit_expr(&args[0]);
                self.write(" as i64; DateTime::<Utc>::from_timestamp(__ts, 0).map(|dt| dt.format(\"%Y-%m-%dT%H:%M:%S\").to_string()).unwrap_or_default() }");
            }
            "localtime" => {
                self.write("{ use chrono::{DateTime, Utc, Local}; let __ts = ");
                self.emit_expr(&args[0]);
                self.write(" as i64; DateTime::<Utc>::from_timestamp(__ts, 0).map(|dt| dt.with_timezone(&Local).format(\"%Y-%m-%dT%H:%M:%S\").to_string()).unwrap_or_default() }");
            }
            // sifr.sys extras
            "sys_exit" => {
                self.write("{ std::process::exit(");
                self.emit_expr(&args[0]);
                self.write(" as i32) }");
            }
            "sys_version" => {
                self.write("\"sifr 0.1.0\".to_string()");
            }
            "sys_platform" => {
                self.write("std::env::consts::OS.to_string()");
            }
            "sys_maxsize" => {
                self.write("i64::MAX");
            }
            "subprocess_run" => {
                self.write("(|| -> Result<String, IOError> { let output = std::process::Command::new(\"sh\").args([\"-c\", ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("]).output().map_err(__io_err)?; Ok(String::from_utf8_lossy(&output.stdout).trim().to_string()) })()");
            }
            "subprocess_run_with_input" => {
                self.write("(|| -> Result<String, IOError> { use std::io::Write; let mut child = std::process::Command::new(\"sh\").args([\"-c\", ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("]).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).spawn().map_err(__io_err)?; if let Some(mut stdin) = child.stdin.take() { stdin.write_all(");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(".as_bytes()).map_err(__io_err)?; } let output = child.wait_with_output().map_err(__io_err)?; Ok(String::from_utf8_lossy(&output.stdout).trim().to_string()) })()");
            }
            "subprocess_run_structured" => {
                self.write("(|| -> Result<Vec<String>, IOError> { let output = std::process::Command::new(\"sh\").args([\"-c\", ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("]).output().map_err(__io_err)?; let stdout = String::from_utf8_lossy(&output.stdout).to_string(); let stderr = String::from_utf8_lossy(&output.stderr).to_string(); let returncode = output.status.code().unwrap_or(-1).to_string(); Ok(vec![stdout, stderr, returncode]) })()");
            }
            // sifr.html
            "html_escape" => {
                self.write("{ let __s = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; __s.replace('&', \"&amp;\").replace('<', \"&lt;\").replace('>', \"&gt;\").replace('\"', \"&quot;\").replace('\\'', \"&#x27;\") }");
            }
            "html_unescape" => {
                self.write("{ let __s = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; __s.replace(\"&amp;\", \"&\").replace(\"&lt;\", \"<\").replace(\"&gt;\", \">\").replace(\"&quot;\", \"\\\"\").replace(\"&#x27;\", \"'\").replace(\"&#39;\", \"'\") }");
            }
            // sifr.calendar
            "calendar_isleap" => {
                self.write("{ let __y = ");
                self.emit_expr(&args[0]);
                self.write("; (__y % 4 == 0 && __y % 100 != 0) || (__y % 400 == 0) }");
            }
            "calendar_weekday" => {
                // Tomohiko Sakamoto's algorithm for day of week (0=Monday)
                self.write("{ let __y0 = ");
                self.emit_expr(&args[0]);
                self.write("; let __m0 = ");
                self.emit_expr(&args[1]);
                self.write("; let __d0 = ");
                self.emit_expr(&args[2]);
                self.write("; let __t = [0i64, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4]; let __y = if __m0 < 3 { __y0 - 1 } else { __y0 }; ((__y + __y/4 - __y/100 + __y/400 + __t[(__m0-1) as usize] + __d0) % 7 + 6) % 7 }");
            }
            "calendar_monthrange" => {
                self.write("{ let __y = ");
                self.emit_expr(&args[0]);
                self.write("; let __m = ");
                self.emit_expr(&args[1]);
                self.write("; let __days = match __m { 1|3|5|7|8|10|12 => 31i64, 4|6|9|11 => 30, 2 => if (__y%4==0 && __y%100!=0)||(__y%400==0) { 29 } else { 28 }, _ => 30 }; let __t = [0i64,3,2,5,0,3,5,1,4,6,2,4]; let __y2 = if __m < 3 { __y-1 } else { __y }; let __wd = ((__y2+__y2/4-__y2/100+__y2/400+__t[(__m-1) as usize]+1)%7+6)%7; vec![__wd, __days] }");
            }
            // sifr.gzip
            "gzip_compress" => {
                self.write("{ use std::io::Write; let __data = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(".as_bytes(); let mut __enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default()); __enc.write_all(__data).unwrap_or(()); __enc.finish().unwrap_or_default().iter().map(|b| *b as i64).collect::<Vec<i64>>() }");
            }
            "gzip_decompress" => {
                self.write(
                    "(|| -> Result<String, IOError> { use std::io::Read; let __bytes: Vec<u8> = ",
                );
                self.emit_expr(&args[0]);
                self.write(".iter().map(|b| *b as u8).collect(); let mut __dec = flate2::read::GzDecoder::new(__bytes.as_slice()); let mut __out = String::new(); __dec.read_to_string(&mut __out).map_err(__io_err)?; Ok(__out) })()");
            }
            // sifr.zipfile
            "zip_create" => {
                self.write("(|| -> Result<(), IOError> { let __f = std::fs::File::create(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map_err(__io_err)?; drop(zip::ZipWriter::new(__f)); Ok(()) })()");
            }
            "zip_add_file" => {
                self.write("(|| -> Result<(), IOError> { let __path = ");
                self.emit_expr_as_str_ref(&args[0]);
                self.write("; let __name = ");
                self.emit_expr_as_str_ref(&args[1]);
                self.write("; let __content = ");
                self.emit_expr_as_str_ref(&args[2]);
                self.write("; let __f = std::fs::OpenOptions::new().read(true).write(true).open(__path).map_err(__io_err)?; let mut __zip = zip::ZipWriter::new_append(__f).map_err(|e| IOError::new(e.to_string()))?; let __opts = zip::write::FileOptions::default(); __zip.start_file(__name, __opts).map_err(|e| IOError::new(e.to_string()))?; use std::io::Write; __zip.write_all(__content.as_bytes()).map_err(__io_err)?; __zip.finish().map_err(|e| IOError::new(e.to_string()))?; Ok(()) })()");
            }
            "zip_read_file" => {
                self.write("(|| -> Result<String, IOError> { let __f = std::fs::File::open(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map_err(__io_err)?; let mut __zip = zip::ZipArchive::new(__f).map_err(|e| IOError::new(e.to_string()))?; let mut __file = __zip.by_name(");
                self.emit_expr_as_str_ref(&args[1]);
                self.write(").map_err(|e| IOError::new(e.to_string()))?; let mut __content = String::new(); use std::io::Read; __file.read_to_string(&mut __content).map_err(__io_err)?; Ok(__content) })()");
            }
            "zip_namelist" => {
                self.write("(|| -> Result<Vec<String>, IOError> { let __f = std::fs::File::open(");
                self.emit_expr_as_str_ref(&args[0]);
                self.write(").map_err(__io_err)?; let mut __zip = zip::ZipArchive::new(__f).map_err(|e| IOError::new(e.to_string()))?; Ok((0..__zip.len()).map(|i| __zip.by_index(i).map(|f| f.name().to_string()).unwrap_or_default()).collect()) })()");
            }
            // sifr.logging
            "set_global_level" => {
                self.runtime_needs.needs_logging_state = true;
                self.write("{ *__SIFR_GLOBAL_LOG_LEVEL.lock().unwrap() = ");
                self.emit_expr(&args[0]);
                self.write("; }");
            }
            "get_global_level" => {
                self.runtime_needs.needs_logging_state = true;
                self.write("*__SIFR_GLOBAL_LOG_LEVEL.lock().unwrap()");
            }
            _ => {
                // Unknown stdlib function — emit as regular call
                self.write(func);
                self.write("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.emit_expr(arg);
                }
                self.write(")");
            }
        }
    }

    pub(crate) fn try_emit_intrinsic_via_registry(&mut self, func: &str, args: &[HirExpr]) -> bool {
        let rendered_args = args
            .iter()
            .map(|arg| self.render_expr_with_lowered_fallback(arg))
            .collect::<Vec<_>>();
        let Some(lowered) = intrinsics::lower_intrinsic(func, &rendered_args) else {
            return false;
        };

        if let Some(required_crate) = lowered.required_crate {
            self.intrinsic_registry_crates
                .insert(required_crate.to_string());
        }
        for required_crate in lowered.additional_required_crates {
            self.intrinsic_registry_crates
                .insert((*required_crate).to_string());
        }

        self.write(&crate::render_expr(&lowered.expr));
        true
    }

    pub(crate) fn try_emit_method_via_registry(
        &mut self,
        object_ty: &Type,
        object: &HirExpr,
        method: &str,
        args: &[HirExpr],
    ) -> bool {
        let is_deque_data_field = self.is_deque_data_field(object);
        let rendered_object = self.render_expr_with_lowered_fallback(object);
        let mut rendered_args = args
            .iter()
            .map(|arg| self.render_expr_with_lowered_fallback(arg))
            .collect::<Vec<_>>();

        if matches!(object_ty, Type::List(_))
            && matches!(method, "append" | "appendleft")
            && !args.is_empty()
        {
            // Preserve legacy behavior: clone TypeVar list args to avoid move issues.
            if matches!(args[0].ty(), Type::TypeVar(_)) {
                rendered_args[0] = format!("{}.clone()", rendered_args[0]);
            }
        }

        if matches!(object_ty, Type::List(_)) && method == "insert" && args.len() >= 2 {
            // Preserve legacy behavior: clone borrowed/mut-borrowed move-owned values.
            let needs_clone = if let HirExpr::Name { name, ty } = &args[1] {
                (self.borrowed_params.contains(name.as_str())
                    || self.mut_borrowed_params.contains(name.as_str()))
                    && ty.ownership() != sifr_type_system::OwnershipKind::Copy
            } else {
                false
            };
            if needs_clone {
                rendered_args[1] = format!("{}.clone()", rendered_args[1]);
            }
        }

        let Some(lowered) = methods::lower_method_with_context(
            object_ty,
            method,
            &rendered_object,
            &rendered_args,
            is_deque_data_field,
        ) else {
            return false;
        };
        self.write(&crate::render_expr(&lowered.expr));
        true
    }
}
