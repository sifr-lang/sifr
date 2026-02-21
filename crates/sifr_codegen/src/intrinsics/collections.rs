//! Collections intrinsic lowerers for registry migration.

use crate::RustExpr;

fn cloned_vec(expr: &str) -> String {
    format!("({expr}).clone()")
}

fn borrowed_str(expr: &str) -> String {
    format!("&({expr})")
}

pub(super) fn lower_new_set(args: &[String]) -> Option<RustExpr> {
    if !args.is_empty() {
        return None;
    }
    Some(RustExpr::RawCode("Vec::<i64>::new()".to_string()))
}

pub(super) fn lower_set_from_list(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let mut s = {}; s.sort(); s.dedup(); s }}",
        cloned_vec(&args[0])
    )))
}

pub(super) fn lower_set_add(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let mut s = {}; let v = {}; if !s.contains(&v) {{ s.push(v); }} s }}",
        cloned_vec(&args[0]),
        args[1]
    )))
}

pub(super) fn lower_set_contains(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{}.contains(&{})",
        args[0], args[1]
    )))
}

pub(super) fn lower_set_remove(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let mut s = {}; s.retain(|x| *x != {}); s }}",
        cloned_vec(&args[0]),
        args[1]
    )))
}

pub(super) fn lower_set_len(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::Cast {
        expr: Box::new(RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Ident(args[0].clone())),
            method: "len".to_string(),
            args: vec![],
        }),
        ty: crate::RustType::I64,
    })
}

pub(super) fn lower_set_union(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let mut s = {}; for v in {}.iter() {{ if !s.contains(v) {{ s.push(*v); }} }} s.sort(); s }}",
        cloned_vec(&args[0]),
        args[1]
    )))
}

pub(super) fn lower_set_intersection(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let __a = {}; let __b = {}; __a.iter().filter(|x| __b.contains(x)).cloned().collect::<Vec<i64>>() }}",
        cloned_vec(&args[0]),
        cloned_vec(&args[1])
    )))
}

pub(super) fn lower_counter_from_list(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let mut counts = std::collections::HashMap::<String, i64>::new(); for item in {}.iter() {{ *counts.entry(item.clone()).or_insert(0) += 1; }} serde_json::to_string(&counts).unwrap_or_default() }}",
        args[0]
    )))
}

pub(super) fn lower_counter_get(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let data: std::collections::HashMap<String, i64> = serde_json::from_str({}).unwrap_or_default(); let __key = {}; *data.get(__key.as_str()).unwrap_or(&0) }}",
        borrowed_str(&args[0]),
        args[1]
    )))
}

pub(super) fn lower_counter_most_common(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let data: std::collections::HashMap<String, i64> = serde_json::from_str({}).unwrap_or_default(); let mut pairs: Vec<(String, i64)> = data.into_iter().collect(); pairs.sort_by(|a, b| b.1.cmp(&a.1)); pairs.truncate({} as usize); let items: Vec<String> = pairs.iter().map(|(k, v)| format!(\"[\\\"{{}}\\\",{{}}]\", k, v)).collect(); format!(\"[{{}}]\", items.join(\",\")) }}",
        borrowed_str(&args[0]),
        args[1]
    )))
}

pub(super) fn lower_counter_total(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let data: std::collections::HashMap<String, i64> = serde_json::from_str({}).unwrap_or_default(); data.values().sum::<i64>() }}",
        borrowed_str(&args[0])
    )))
}

pub(super) fn lower_counter_values(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let data: std::collections::HashMap<String, i64> = serde_json::from_str({}).unwrap_or_default(); data.values().cloned().collect::<Vec<i64>>() }}",
        borrowed_str(&args[0])
    )))
}

pub(super) fn lower_counter_keys(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let data: std::collections::HashMap<String, i64> = serde_json::from_str({}).unwrap_or_default(); data.keys().cloned().collect::<Vec<String>>() }}",
        borrowed_str(&args[0])
    )))
}

pub(super) fn lower_counter_items(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let data: std::collections::HashMap<String, i64> = serde_json::from_str({}).unwrap_or_default(); let mut pairs: Vec<(String, i64)> = data.into_iter().collect(); pairs.sort_by(|a, b| a.0.cmp(&b.0)); let items: Vec<String> = pairs.iter().map(|(k, v)| format!(\"[\\\"{{}}\\\",{{}}]\", k, v)).collect(); format!(\"[{{}}]\", items.join(\",\")) }}",
        borrowed_str(&args[0])
    )))
}

pub(super) fn lower_counter_increment(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let mut data: std::collections::HashMap<String, i64> = serde_json::from_str({}).unwrap_or_default(); *data.entry({}.to_string()).or_insert(0) += 1; serde_json::to_string(&data).unwrap_or_default() }}",
        borrowed_str(&args[0]),
        args[1]
    )))
}

pub(super) fn lower_defaultdict_new(args: &[String]) -> Option<RustExpr> {
    if args.len() != 1 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "format!(\"{{\\\"__default__\\\":{{}}}}\", {})",
        args[0]
    )))
}

pub(super) fn lower_defaultdict_get(args: &[String]) -> Option<RustExpr> {
    if args.len() != 2 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let data: std::collections::HashMap<String, i64> = serde_json::from_str({}).unwrap_or_default(); let def = data.get(\"__default__\").cloned().unwrap_or(0); *data.get({}).unwrap_or(&def) }}",
        borrowed_str(&args[0]),
        borrowed_str(&args[1])
    )))
}

pub(super) fn lower_defaultdict_set(args: &[String]) -> Option<RustExpr> {
    if args.len() != 3 {
        return None;
    }
    Some(RustExpr::RawCode(format!(
        "{{ let mut data: std::collections::HashMap<String, serde_json::Value> = serde_json::from_str({}).unwrap_or_default(); data.insert({}.to_string(), serde_json::json!({})); serde_json::to_string(&data).unwrap_or_default() }}",
        borrowed_str(&args[0]),
        args[1],
        args[2]
    )))
}
