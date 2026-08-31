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
}
pub use __sifr_project_unions::__SifrUnion_8_x3asequence5_x3aunion1_x3a211_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr;
use ::sifr_runtime::SifrInt;

fn create_user(id: SifrInt, name: &str) -> SifrInt {
    id.clone()
}

fn handle_command(cmd: &str) -> String {
    if cmd == "start" {
        return "Starting...".to_string();
    } else {
        return "Unknown command".to_string();
    }
}

fn describe(x: &__SifrUnion_8_x3asequence5_x3aunion1_x3a211_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr) -> String {
    match x {
        __SifrUnion_8_x3asequence5_x3aunion1_x3a211_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr::__SifrUnionVariant_4_x3aatom3_x3aint(x) => {
            return format!("number: {}", x + &SifrInt::from_i64(1));
        },
        __SifrUnion_8_x3asequence5_x3aunion1_x3a211_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr::__SifrUnionVariant_4_x3aatom3_x3astr(x) => {
            return format!("text: {}", x);
        },
    }
}

fn find_user(name: &str) -> Option<String> {
    if name == "alice" {
        return Some("Alice Smith".to_string());
    }
    None
}

fn main() {
    let uid: SifrInt = create_user(SifrInt::from_i64(42), &"alice".to_string());
    println!("{}", uid);
    println!("{}", handle_command(&"start".to_string()));
    println!("{}", handle_command(&"stop".to_string()));
    println!("{}", describe(&__SifrUnion_8_x3asequence5_x3aunion1_x3a211_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr::__SifrUnionVariant_4_x3aatom3_x3aint(SifrInt::from_i64(42))));
    println!("{}", describe(&__SifrUnion_8_x3asequence5_x3aunion1_x3a211_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr::__SifrUnionVariant_4_x3aatom3_x3astr("hello".to_string().to_owned())));
    let user: Option<String> = find_user(&"alice".to_string());
    if let Some(user) = user {
        println!("{}", user);
    } else {
        println!("not found");
    }
}
