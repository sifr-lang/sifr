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
                Self::SifrGeneratedUnionVariant4X3aatom3X3aint(v) => write!(f, "{v}"),
                Self::SifrGeneratedUnionVariant4X3aatom3X3astr(v) => write!(f, "{v}"),
            }
        }
    }
}
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr;
fn create_user(id: &SifrInt, _: &str) -> SifrInt {
    (*id).clone()
}
fn handle_command(cmd: &str) -> String {
    if cmd == "start" {
        "Starting...".to_string()
    } else {
        "Unknown command".to_string()
    }
}
fn describe(
    x: &SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr,
) -> String {
    match x {
        SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr::SifrGeneratedUnionVariant4X3aatom3X3aint(
            x,
        ) => format!("number: {}", ::std::ops::Add::add(x, & SifrInt::from_i64(1))),
        SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr::SifrGeneratedUnionVariant4X3aatom3X3astr(
            x,
        ) => format!("text: {x}"),
    }
}
fn find_user(name: &str) -> Option<String> {
    if name == "alice" {
        return Some("Alice Smith".to_string());
    }
    None
}
fn main() {
    let uid: SifrInt = create_user(&SifrInt::from_i64(42), "alice");
    println!("{uid}");
    println!("{}", handle_command("start"));
    println!("{}", handle_command("stop"));
    println!(
        "{}", describe(&
        SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr::SifrGeneratedUnionVariant4X3aatom3X3aint(SifrInt::from_i64(42)))
    );
    println!(
        "{}", describe(&
        SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr::SifrGeneratedUnionVariant4X3aatom3X3astr("hello"
        .to_string()))
    );
    let user: Option<String> = find_user("alice");
    if let Some(user) = user {
        println!("{user}");
    } else {
        println!("not found");
    }
}
