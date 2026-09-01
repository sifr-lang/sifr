// src/main.rs
mod sifr_generated_project_unions {
    use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr {
        SifrGeneratedUnionVariant4X3aatom3X3aint(SifrInt),
        SifrGeneratedUnionVariant4X3aatom3X3astr(String),
    }
    impl ::std::fmt::Display
        for SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr
    {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr::SifrGeneratedUnionVariant4X3aatom3X3aint(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr::SifrGeneratedUnionVariant4X3aatom3X3astr(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a311X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool
    {
        SifrGeneratedUnionVariant4X3aatom4X3abool(bool),
        SifrGeneratedUnionVariant4X3aatom3X3aint(SifrInt),
        SifrGeneratedUnionVariant4X3aatom3X3astr(String),
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a311X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a311X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool::SifrGeneratedUnionVariant4X3aatom4X3abool(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a311X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool::SifrGeneratedUnionVariant4X3aatom3X3aint(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a311X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool::SifrGeneratedUnionVariant4X3aatom3X3astr(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
}
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a311X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool;
fn find_user(name: &str) -> Option<String> {
    if name == "alice" {
        return Some("Alice Smith".to_string());
    }
    None
}
fn process(
    x: &SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr,
) -> String {
    match x {
        SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr::SifrGeneratedUnionVariant4X3aatom3X3aint(
            x,
        ) => format!("number: {x}"),
        SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr::SifrGeneratedUnionVariant4X3aatom3X3astr(
            x,
        ) => format!("string: {x}"),
    }
}
fn classify(
    x: &SifrGeneratedUnion8X3asequence5X3aunion1X3a311X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool,
) -> String {
    match x {
        SifrGeneratedUnion8X3asequence5X3aunion1X3a311X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool::SifrGeneratedUnionVariant4X3aatom3X3aint(
            _,
        ) => "int".to_string(),
        SifrGeneratedUnion8X3asequence5X3aunion1X3a311X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool::SifrGeneratedUnionVariant4X3aatom3X3astr(
            _,
        ) => "str".to_string(),
        SifrGeneratedUnion8X3asequence5X3aunion1X3a311X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool::SifrGeneratedUnionVariant4X3aatom4X3abool(
            _,
        ) => "bool".to_string(),
    }
}
fn process_optional(x: &Option<String>) -> String {
    let Some(x) = x.as_ref() else {
        return "none".to_string();
    };
    x.to_uppercase()
}
const fn consume(s: String) -> String {
    s
}
fn main() {
    let result: Option<String> = find_user(&"alice".to_string());
    if let Some(result) = result {
        println!("{result}");
    }
    let missing: Option<String> = find_user(&"bob".to_string());
    if missing.is_none() {
        println!("not found");
    }
    println!(
        "{}", process(&
        SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr::SifrGeneratedUnionVariant4X3aatom3X3aint(SifrInt::from_i64(42)))
    );
    println!(
        "{}", process(&
        SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr::SifrGeneratedUnionVariant4X3aatom3X3astr("hello"
        .to_string().to_owned()))
    );
    println!(
        "{}", classify(&
        SifrGeneratedUnion8X3asequence5X3aunion1X3a311X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool::SifrGeneratedUnionVariant4X3aatom3X3aint(SifrInt::from_i64(1)))
    );
    println!(
        "{}", classify(&
        SifrGeneratedUnion8X3asequence5X3aunion1X3a311X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool::SifrGeneratedUnionVariant4X3aatom3X3astr("hi"
        .to_string().to_owned()))
    );
    println!(
        "{}", classify(&
        SifrGeneratedUnion8X3asequence5X3aunion1X3a311X3a4X3aatom3X3aint11X3a4X3aatom3X3astr12X3a4X3aatom4X3abool::SifrGeneratedUnionVariant4X3aatom4X3abool(true))
    );
    println!(
        "{}",
        process_optional(&Some("world".to_string().to_owned()))
    );
    println!("{}", process_optional(&None));
    let mut s: String = "hello".to_string();
    let x: String = consume(s);
    s = "world".to_string();
    println!("{s}");
    println!("{x}");
}
