//! IR-backed helpers for generating codegen preamble items.

use crate::{
    RustExpr, RustItem, RustParam, RustStmt, RustType, Type, Visibility,
};

pub fn sifr_type_to_rust_type(ty: &Type) -> RustType {
    match ty {
        Type::Int | Type::LiteralInt(_) => RustType::I64,
        Type::Float => RustType::F64,
        Type::Bool | Type::LiteralBool(_) => RustType::Bool,
        Type::Str | Type::LiteralStr(_) => RustType::String_,
        Type::None => RustType::Unit,
        Type::List(inner) => RustType::Vec(Box::new(sifr_type_to_rust_type(inner))),
        Type::Dict(key, value) => RustType::HashMap(
            Box::new(sifr_type_to_rust_type(key)),
            Box::new(sifr_type_to_rust_type(value)),
        ),
        Type::Set(inner) => RustType::HashSet(Box::new(sifr_type_to_rust_type(inner))),
        Type::Tuple(items) => RustType::Tuple(items.iter().map(sifr_type_to_rust_type).collect()),
        Type::Result(ok, err) => RustType::Result(
            Box::new(sifr_type_to_rust_type(ok)),
            Box::new(sifr_type_to_rust_type(err)),
        ),
        Type::Union(members) => {
            let non_none: Vec<&Type> = members
                .iter()
                .filter(|m| !matches!(m, Type::None))
                .collect();
            let has_none = members.iter().any(|m| matches!(m, Type::None));
            if has_none && non_none.len() == 1 {
                RustType::Option(Box::new(sifr_type_to_rust_type(non_none[0])))
            } else {
                RustType::Named(ty.rust_type())
            }
        }
        _ => RustType::RawCode(ty.rust_type()),
    }
}

pub fn build_error_type_items(
    name: &str,
    extra_fields: &[(String, RustType)],
    constructor_defaults: &[(String, RustExpr)],
) -> Vec<RustItem> {
    let mut fields = vec![("message".to_string(), RustType::String_)];
    fields.extend(extra_fields.iter().cloned());

    let mut init_fields = vec![("message".to_string(), RustExpr::Ident("message".to_string()))];
    init_fields.extend(constructor_defaults.iter().cloned());

    vec![
        RustItem::Struct {
            name: name.to_string(),
            visibility: Visibility::Private,
            derives: vec!["Debug".to_string(), "Clone".to_string()],
            fields,
        },
        RustItem::Impl {
            target: name.to_string(),
            type_params: vec![],
            trait_: None,
            items: vec![RustItem::Fn {
                name: "new".to_string(),
                visibility: Visibility::Private,
                type_params: vec![],
                params: vec![RustParam::Named {
                    name: "message".to_string(),
                    ty: RustType::String_,
                }],
                ret: Some(RustType::Named("Self".to_string())),
                body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                    name: "Self".to_string(),
                    fields: init_fields,
                }))],
                is_async: false,
            }],
        },
        RustItem::Impl {
            target: name.to_string(),
            type_params: vec![],
            trait_: Some("std::fmt::Display".to_string()),
            items: vec![RustItem::Fn {
                name: "fmt".to_string(),
                visibility: Visibility::Private,
                type_params: vec![],
                params: vec![
                    RustParam::SelfParam { mutable: false },
                    RustParam::Named {
                        name: "f".to_string(),
                        ty: RustType::RawCode("&mut std::fmt::Formatter<'_>".to_string()),
                    },
                ],
                ret: Some(RustType::RawCode("std::fmt::Result".to_string())),
                body: vec![RustStmt::RawCode("write!(f, \"{}\", self.message)".to_string())],
                is_async: false,
            }],
        },
        RustItem::Impl {
            target: name.to_string(),
            type_params: vec![],
            trait_: Some("std::error::Error".to_string()),
            items: vec![],
        },
    ]
}

pub fn build_io_error_items() -> Vec<RustItem> {
    let mut items = build_error_type_items(
        "IOError",
        &[("kind".to_string(), RustType::String_)],
        &[(
            "kind".to_string(),
            RustExpr::Literal(crate::RustLiteral::Str("Other".to_string())),
        )],
    );

    items.push(RustItem::Fn {
        name: "__io_err".to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params: vec![RustParam::Named {
            name: "e".to_string(),
            ty: RustType::RawCode("std::io::Error".to_string()),
        }],
        ret: Some(RustType::Named("IOError".to_string())),
        body: vec![
            RustStmt::Let {
                mutable: false,
                name: "msg".to_string(),
                ty: None,
                value: RustExpr::MethodCall {
                    receiver: Box::new(RustExpr::Ident("e".to_string())),
                    method: "to_string".to_string(),
                    args: vec![],
                },
            },
            RustStmt::Let {
                mutable: false,
                name: "kind".to_string(),
                ty: None,
                value: RustExpr::RawCode(
                    "match e.kind() {
        std::io::ErrorKind::NotFound => \"FileNotFound\",
        std::io::ErrorKind::PermissionDenied => \"PermissionDenied\",
        std::io::ErrorKind::AlreadyExists => \"FileExists\",
        _ => \"Other\",
    }"
                    .to_string(),
                ),
            },
            RustStmt::Return(Some(RustExpr::StructInit {
                name: "IOError".to_string(),
                fields: vec![
                    ("message".to_string(), RustExpr::Ident("msg".to_string())),
                    (
                        "kind".to_string(),
                        RustExpr::MethodCall {
                            receiver: Box::new(RustExpr::Ident("kind".to_string())),
                            method: "to_string".to_string(),
                            args: vec![],
                        },
                    ),
                ],
            })),
        ],
        is_async: false,
    });

    items
}

pub fn build_file_handle_infra_items() -> Vec<RustItem> {
    vec![
        RustItem::RawCode(
            "enum SifrFileHandle {
    TextRead(std::io::BufReader<std::fs::File>),
    TextWrite(std::io::BufWriter<std::fs::File>),
    BinaryRead(std::io::BufReader<std::fs::File>),
    BinaryWrite(std::io::BufWriter<std::fs::File>),
}"
            .to_string(),
        ),
        RustItem::Static {
            name: "__SIFR_FILE_HANDLES".to_string(),
            visibility: Visibility::Private,
            ty: RustType::RawCode(
                "std::sync::LazyLock<Mutex<HashMap<i64, SifrFileHandle>>>".to_string(),
            ),
            value: RustExpr::RawCode("std::sync::LazyLock::new(|| Mutex::new(HashMap::new()))".to_string()),
        },
    ]
}

pub fn build_file_handle_struct_items() -> Vec<RustItem> {
    vec![
        RustItem::Struct {
            name: "FileHandle".to_string(),
            visibility: Visibility::Private,
            derives: vec!["Debug".to_string(), "Clone".to_string()],
            fields: vec![
                ("_handle".to_string(), RustType::I64),
                ("_mode".to_string(), RustType::String_),
            ],
        },
        RustItem::Impl {
            target: "FileHandle".to_string(),
            type_params: vec![],
            trait_: None,
            items: vec![
                RustItem::Fn {
                    name: "new".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![
                        RustParam::Named {
                            name: "_handle".to_string(),
                            ty: RustType::I64,
                        },
                        RustParam::Named {
                            name: "_mode".to_string(),
                            ty: RustType::String_,
                        },
                    ],
                    ret: Some(RustType::Named("Self".to_string())),
                    body: vec![RustStmt::Return(Some(RustExpr::StructInit {
                        name: "Self".to_string(),
                        fields: vec![
                            ("_handle".to_string(), RustExpr::Ident("_handle".to_string())),
                            ("_mode".to_string(), RustExpr::Ident("_mode".to_string())),
                        ],
                    }))],
                    is_async: false,
                },
                raw_file_handle_method(
                    "read",
                    vec![RustParam::SelfParam { mutable: false }],
                    Some(RustType::Result(
                        Box::new(RustType::String_),
                        Box::new(RustType::Named("IOError".to_string())),
                    )),
                    "(|| -> Result<String, IOError> { let __hid = self._handle; let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) { Some(SifrFileHandle::TextRead(ref mut __r)) => { use std::io::Read; let mut __s = String::new(); __r.read_to_string(&mut __s).map_err(__io_err)?; Ok(__s) }, _ => Err(IOError { message: \"file not open for reading\".to_string(), kind: \"Other\".to_string() }) } })()",
                ),
                raw_file_handle_method(
                    "write",
                    vec![
                        RustParam::SelfParam { mutable: false },
                        RustParam::Named {
                            name: "data".to_string(),
                            ty: RustType::Ref {
                                mutable: false,
                                inner: Box::new(RustType::String_),
                            },
                        },
                    ],
                    Some(RustType::Result(
                        Box::new(RustType::Unit),
                        Box::new(RustType::Named("IOError".to_string())),
                    )),
                    "(|| -> Result<(), IOError> { let __hid = self._handle; let __data = data; let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) { Some(SifrFileHandle::TextWrite(ref mut __w)) => { use std::io::Write; __w.write_all(__data.as_bytes()).map_err(__io_err)?; Ok(()) }, _ => Err(IOError { message: \"file not open for writing\".to_string(), kind: \"Other\".to_string() }) } })()",
                ),
                raw_file_handle_method(
                    "readline",
                    vec![RustParam::SelfParam { mutable: false }],
                    Some(RustType::Result(
                        Box::new(RustType::Option(Box::new(RustType::String_))),
                        Box::new(RustType::Named("IOError".to_string())),
                    )),
                    "(|| -> Result<Option<String>, IOError> { let __hid = self._handle; let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) { Some(SifrFileHandle::TextRead(ref mut __r)) => { use std::io::BufRead; let mut __line = String::new(); let __n = __r.read_line(&mut __line).map_err(__io_err)?; if __n == 0 { Ok(None) } else { if __line.ends_with('\\n') { __line.pop(); if __line.ends_with('\\r') { __line.pop(); } } Ok(Some(__line)) } }, _ => Err(IOError { message: \"file not open for reading\".to_string(), kind: \"Other\".to_string() }) } })()",
                ),
                raw_file_handle_method(
                    "readlines",
                    vec![RustParam::SelfParam { mutable: false }],
                    Some(RustType::Result(
                        Box::new(RustType::Vec(Box::new(RustType::String_))),
                        Box::new(RustType::Named("IOError".to_string())),
                    )),
                    "(|| -> Result<Vec<String>, IOError> { let __hid = self._handle; let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) { Some(SifrFileHandle::TextRead(ref mut __r)) => { use std::io::BufRead; let mut __lines: Vec<String> = Vec::new(); let mut __line = String::new(); loop { __line.clear(); let __n = __r.read_line(&mut __line).map_err(__io_err)?; if __n == 0 { break; } let mut __l = __line.clone(); if __l.ends_with('\\n') { __l.pop(); if __l.ends_with('\\r') { __l.pop(); } } __lines.push(__l); } Ok(__lines) }, _ => Err(IOError { message: \"file not open for reading\".to_string(), kind: \"Other\".to_string() }) } })()",
                ),
                RustItem::Fn {
                    name: "close".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfParam { mutable: false }],
                    ret: None,
                    body: vec![RustStmt::RawCode(
                        "let __hid = self._handle; __SIFR_FILE_HANDLES.lock().unwrap().remove(&__hid);"
                            .to_string(),
                    )],
                    is_async: false,
                },
                raw_file_handle_method(
                    "read_bytes",
                    vec![RustParam::SelfParam { mutable: false }],
                    Some(RustType::Result(
                        Box::new(RustType::Vec(Box::new(RustType::I64))),
                        Box::new(RustType::Named("IOError".to_string())),
                    )),
                    "(|| -> Result<Vec<i64>, IOError> { let __hid = self._handle; let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) { Some(SifrFileHandle::BinaryRead(ref mut __r)) => { use std::io::Read; let mut __buf = Vec::new(); __r.read_to_end(&mut __buf).map_err(__io_err)?; Ok(__buf.into_iter().map(|b| b as i64).collect()) }, _ => Err(IOError { message: \"file not open for binary reading\".to_string(), kind: \"Other\".to_string() }) } })()",
                ),
                raw_file_handle_method(
                    "write_bytes",
                    vec![
                        RustParam::SelfParam { mutable: false },
                        RustParam::Named {
                            name: "data".to_string(),
                            ty: RustType::Ref {
                                mutable: false,
                                inner: Box::new(RustType::Vec(Box::new(RustType::I64))),
                            },
                        },
                    ],
                    Some(RustType::Result(
                        Box::new(RustType::Unit),
                        Box::new(RustType::Named("IOError".to_string())),
                    )),
                    "(|| -> Result<(), IOError> { let __hid = self._handle; let __data = data; let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap(); match __handles.get_mut(&__hid) { Some(SifrFileHandle::BinaryWrite(ref mut __w)) => { use std::io::Write; let __bytes: Vec<u8> = __data.iter().map(|&b| b as u8).collect(); __w.write_all(&__bytes).map_err(__io_err)?; Ok(()) }, _ => Err(IOError { message: \"file not open for binary writing\".to_string(), kind: \"Other\".to_string() }) } })()",
                ),
                RustItem::Fn {
                    name: "__enter__".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfParam { mutable: false }],
                    ret: Some(RustType::Ref {
                        mutable: false,
                        inner: Box::new(RustType::Named("Self".to_string())),
                    }),
                    body: vec![RustStmt::Return(Some(RustExpr::Ident("self".to_string())))],
                    is_async: false,
                },
                RustItem::Fn {
                    name: "__exit__".to_string(),
                    visibility: Visibility::Private,
                    type_params: vec![],
                    params: vec![RustParam::SelfParam { mutable: false }],
                    ret: None,
                    body: vec![RustStmt::Expr(RustExpr::MethodCall {
                        receiver: Box::new(RustExpr::Ident("self".to_string())),
                        method: "close".to_string(),
                        args: vec![],
                    })],
                    is_async: false,
                },
            ],
        },
    ]
}

pub fn build_logging_items() -> Vec<RustItem> {
    vec![RustItem::Static {
        name: "__SIFR_GLOBAL_LOG_LEVEL".to_string(),
        visibility: Visibility::Private,
        ty: RustType::RawCode("std::sync::LazyLock<Mutex<i64>>".to_string()),
        value: RustExpr::RawCode("std::sync::LazyLock::new(|| Mutex::new(20))".to_string()),
    }]
}

fn raw_file_handle_method(
    name: &str,
    params: Vec<RustParam>,
    ret: Option<RustType>,
    body_expr: &str,
) -> RustItem {
    RustItem::Fn {
        name: name.to_string(),
        visibility: Visibility::Private,
        type_params: vec![],
        params,
        ret,
        body: vec![RustStmt::Return(Some(RustExpr::RawCode(body_expr.to_string())))],
        is_async: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_items;

    #[test]
    fn maps_types_to_structured_rust_types() {
        assert_eq!(sifr_type_to_rust_type(&Type::Int), RustType::I64);
        assert_eq!(
            sifr_type_to_rust_type(&Type::List(Box::new(Type::Str))),
            RustType::Vec(Box::new(RustType::String_))
        );
        assert_eq!(
            sifr_type_to_rust_type(&Type::Union(vec![Type::Int, Type::None])),
            RustType::Option(Box::new(RustType::I64))
        );
    }

    #[test]
    fn error_items_render_expected_shapes() {
        let items = build_error_type_items(
            "RegexError",
            &[("detail".to_string(), RustType::String_)],
            &[(
                "detail".to_string(),
                RustExpr::RawCode("String::new()".to_string()),
            )],
        );
        let rendered = render_items(&items);
        assert!(rendered.contains("struct RegexError"));
        assert!(rendered.contains("fn new(message: String) -> Self"));
        assert!(rendered.contains("impl std::error::Error for RegexError"));
    }

    #[test]
    fn file_handle_items_render_core_symbols() {
        let mut items = build_file_handle_infra_items();
        items.extend(build_file_handle_struct_items());
        let rendered = render_items(&items);
        assert!(rendered.contains("enum SifrFileHandle"));
        assert!(rendered.contains("static __SIFR_FILE_HANDLES"));
        assert!(rendered.contains("impl FileHandle"));
        assert!(rendered.contains("fn read(&self) -> Result<String, IOError>"));
    }
}
