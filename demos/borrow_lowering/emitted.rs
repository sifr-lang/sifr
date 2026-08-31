// src/main.rs
mod __sifr_project_unions {
    use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a211_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr {
        __SifrUnionVariant_4_x3aatom3_x3aint(SifrInt),
        __SifrUnionVariant_4_x3aatom3_x3astr(String),
    }
    impl ::std::fmt::Display
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a211_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a211_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr::__SifrUnionVariant_4_x3aatom3_x3aint(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a211_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr::__SifrUnionVariant_4_x3aatom3_x3astr(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
            }
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a211_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool {
        __SifrUnionVariant_4_x3aatom4_x3abool(bool),
        __SifrUnionVariant_4_x3aatom3_x3astr(String),
    }
    impl ::std::fmt::Display
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a211_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a211_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool::__SifrUnionVariant_4_x3aatom4_x3abool(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a211_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool::__SifrUnionVariant_4_x3aatom3_x3astr(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
            }
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a311_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool {
        __SifrUnionVariant_4_x3aatom4_x3abool(bool),
        __SifrUnionVariant_4_x3aatom3_x3aint(SifrInt),
        __SifrUnionVariant_4_x3aatom3_x3astr(String),
    }
    impl ::std::fmt::Display
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a311_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a311_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool::__SifrUnionVariant_4_x3aatom4_x3abool(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a311_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool::__SifrUnionVariant_4_x3aatom3_x3aint(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a311_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool::__SifrUnionVariant_4_x3aatom3_x3astr(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
            }
        }
    }
}
pub use __sifr_project_unions::__SifrUnion_8_x3asequence5_x3aunion1_x3a211_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr;
pub use __sifr_project_unions::__SifrUnion_8_x3asequence5_x3aunion1_x3a211_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool;
pub use __sifr_project_unions::__SifrUnion_8_x3asequence5_x3aunion1_x3a311_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool;
use ::sifr_runtime::SifrInt;

fn find_user(name: &str) -> Option<String> {
    if name == "alice" {
        return Some("Alice Smith".to_string());
    }
    None
}

fn process(x: &__SifrUnion_8_x3asequence5_x3aunion1_x3a211_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr) -> String {
    match x {
        __SifrUnion_8_x3asequence5_x3aunion1_x3a211_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr::__SifrUnionVariant_4_x3aatom3_x3aint(x) => {
            return format!("number: {}", x);
        },
        __SifrUnion_8_x3asequence5_x3aunion1_x3a211_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr::__SifrUnionVariant_4_x3aatom3_x3astr(x) => {
            return format!("string: {}", x);
        },
    }
}

fn classify(x: &__SifrUnion_8_x3asequence5_x3aunion1_x3a311_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool) -> String {
    match x {
        __SifrUnion_8_x3asequence5_x3aunion1_x3a311_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool::__SifrUnionVariant_4_x3aatom3_x3aint(x) => {
            return "int".to_string();
        },
        __SifrUnion_8_x3asequence5_x3aunion1_x3a311_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool::__SifrUnionVariant_4_x3aatom3_x3astr(x) => {
            return "str".to_string();
        },
        __SifrUnion_8_x3asequence5_x3aunion1_x3a311_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool::__SifrUnionVariant_4_x3aatom4_x3abool(x) => {
            return "bool".to_string();
        },
    }
}

fn process_optional(x: &Option<String>) -> String {
    if let Some(x) = x.as_ref() {
        return x.to_uppercase();
    }
    "none".to_string()
}

fn consume(s: String) -> String {
    s
}

fn main() {
    let result: Option<String> = find_user(&"alice".to_string());
    if let Some(result) = result {
        println!("{}", result);
    }
    let missing: Option<String> = find_user(&"bob".to_string());
    if missing.is_none() {
        println!("not found");
    }
    println!("{}", process(&__SifrUnion_8_x3asequence5_x3aunion1_x3a211_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr::__SifrUnionVariant_4_x3aatom3_x3aint(SifrInt::from_i64(42))));
    println!("{}", process(&__SifrUnion_8_x3asequence5_x3aunion1_x3a211_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr::__SifrUnionVariant_4_x3aatom3_x3astr("hello".to_string().to_owned())));
    println!("{}", classify(&__SifrUnion_8_x3asequence5_x3aunion1_x3a311_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool::__SifrUnionVariant_4_x3aatom3_x3aint(SifrInt::from_i64(1))));
    println!("{}", classify(&__SifrUnion_8_x3asequence5_x3aunion1_x3a311_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool::__SifrUnionVariant_4_x3aatom3_x3astr("hi".to_string().to_owned())));
    println!("{}", classify(&__SifrUnion_8_x3asequence5_x3aunion1_x3a311_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool::__SifrUnionVariant_4_x3aatom4_x3abool(true)));
    println!("{}", process_optional(&Some("world".to_string().to_owned())));
    println!("{}", process_optional(&None));
    let mut s: String = "hello".to_string();
    let x: String = consume(s);
    s = "world".to_string();
    println!("{}", s);
    println!("{}", x);
}
