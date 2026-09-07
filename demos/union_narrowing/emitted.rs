// src/main.rs
mod sifr_generated_project_unions {
    #[derive(Debug, Clone, PartialEq)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a320X3a5X3aclass8X3amainX2eCat1X3a020X3a5X3aclass8X3amainX2eDog1X3a021X3a5X3aclass9X3amainX2eBird1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass9X3amainX2eBird1X3a0(crate::Bird),
        SifrGeneratedUnionVariant5X3aclass8X3amainX2eCat1X3a0(crate::Cat),
        SifrGeneratedUnionVariant5X3aclass8X3amainX2eDog1X3a0(crate::Dog),
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a320X3a5X3aclass8X3amainX2eCat1X3a020X3a5X3aclass8X3amainX2eDog1X3a021X3a5X3aclass9X3amainX2eBird1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a320X3a5X3aclass8X3amainX2eCat1X3a020X3a5X3aclass8X3amainX2eDog1X3a021X3a5X3aclass9X3amainX2eBird1X3a0::SifrGeneratedUnionVariant5X3aclass9X3amainX2eBird1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a320X3a5X3aclass8X3amainX2eCat1X3a020X3a5X3aclass8X3amainX2eDog1X3a021X3a5X3aclass9X3amainX2eBird1X3a0::SifrGeneratedUnionVariant5X3aclass8X3amainX2eCat1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a320X3a5X3aclass8X3amainX2eCat1X3a020X3a5X3aclass8X3amainX2eDog1X3a021X3a5X3aclass9X3amainX2eBird1X3a0::SifrGeneratedUnionVariant5X3aclass8X3amainX2eDog1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
}
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a320X3a5X3aclass8X3amainX2eCat1X3a020X3a5X3aclass8X3amainX2eDog1X3a021X3a5X3aclass9X3amainX2eBird1X3a0;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Dog {
    name: String,
    breed: String,
}
impl Dog {
    const fn new(name: String, breed: String) -> Self {
        Self { name, breed }
    }
}
impl ::std::fmt::Display for Dog {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Dog(name={}, breed={})", self.name, self.breed)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Cat {
    name: String,
    color: String,
}
impl Cat {
    const fn new(name: String, color: String) -> Self {
        Self { name, color }
    }
}
impl ::std::fmt::Display for Cat {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Cat(name={}, color={})", self.name, self.color)
    }
}
#[derive(Debug, Clone, PartialEq)]
struct Bird {
    name: String,
    wingspan: f64,
}
impl Bird {
    const fn new(name: String, wingspan: f64) -> Self {
        Self { name, wingspan }
    }
}
impl ::std::fmt::Display for Bird {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Bird(name={}, wingspan={})", self.name, self.wingspan)
    }
}
fn route_handler(method: &str) -> String {
    if method == "GET" {
        return "get handler".to_string();
    } else if method == "POST" {
        return "post handler".to_string();
    } else if method == "PUT" {
        return "put handler".to_string();
    }
    "unknown".to_string()
}
fn describe_pet(
    pet: &SifrGeneratedUnion8X3asequence5X3aunion1X3a320X3a5X3aclass8X3amainX2eCat1X3a020X3a5X3aclass8X3amainX2eDog1X3a021X3a5X3aclass9X3amainX2eBird1X3a0,
) -> String {
    match pet {
        SifrGeneratedUnion8X3asequence5X3aunion1X3a320X3a5X3aclass8X3amainX2eCat1X3a020X3a5X3aclass8X3amainX2eDog1X3a021X3a5X3aclass9X3amainX2eBird1X3a0::SifrGeneratedUnionVariant5X3aclass8X3amainX2eDog1X3a0(
            pet,
        ) => format!("{} is a {}", pet.name.clone(), pet.breed.clone()),
        SifrGeneratedUnion8X3asequence5X3aunion1X3a320X3a5X3aclass8X3amainX2eCat1X3a020X3a5X3aclass8X3amainX2eDog1X3a021X3a5X3aclass9X3amainX2eBird1X3a0::SifrGeneratedUnionVariant5X3aclass8X3amainX2eCat1X3a0(
            pet,
        ) => format!("{} is {}", pet.name.clone(), pet.color.clone()),
        SifrGeneratedUnion8X3asequence5X3aunion1X3a320X3a5X3aclass8X3amainX2eCat1X3a020X3a5X3aclass8X3amainX2eDog1X3a021X3a5X3aclass9X3amainX2eBird1X3a0::SifrGeneratedUnionVariant5X3aclass9X3amainX2eBird1X3a0(
            pet,
        ) => format!("{} has wingspan {}", pet.name.clone(), pet.wingspan),
    }
}
fn find_value(x: Option<SifrInt>, target: SifrInt) -> String {
    if x == Some(target.clone()) {
        return "found".to_string();
    }
    "not found".to_string()
}
fn is_positive(x: Option<SifrInt>) -> bool {
    if x > Some(SifrInt::from_i64(0)) {
        return true;
    }
    false
}
fn summarize(items: &[String]) -> String {
    if !!items.is_empty() {
        return "no items".to_string();
    }
    format!("{} items", SifrInt::from(items.len()))
}
fn main() {
    println!("{}", route_handler(&"GET".to_string()));
    println!("{}", route_handler(&"POST".to_string()));
    println!("{}", route_handler(&"PUT".to_string()));
    println!("{}", route_handler(&"DELETE".to_string()));
    println!(
        "{}", describe_pet(&
        SifrGeneratedUnion8X3asequence5X3aunion1X3a320X3a5X3aclass8X3amainX2eCat1X3a020X3a5X3aclass8X3amainX2eDog1X3a021X3a5X3aclass9X3amainX2eBird1X3a0::SifrGeneratedUnionVariant5X3aclass8X3amainX2eDog1X3a0(Dog::new("Rex"
        .to_string(), "Labrador".to_string()).clone()))
    );
    println!(
        "{}", describe_pet(&
        SifrGeneratedUnion8X3asequence5X3aunion1X3a320X3a5X3aclass8X3amainX2eCat1X3a020X3a5X3aclass8X3amainX2eDog1X3a021X3a5X3aclass9X3amainX2eBird1X3a0::SifrGeneratedUnionVariant5X3aclass8X3amainX2eCat1X3a0(Cat::new("Whiskers"
        .to_string(), "orange".to_string()).clone()))
    );
    println!(
        "{}", describe_pet(&
        SifrGeneratedUnion8X3asequence5X3aunion1X3a320X3a5X3aclass8X3amainX2eCat1X3a020X3a5X3aclass8X3amainX2eDog1X3a021X3a5X3aclass9X3amainX2eBird1X3a0::SifrGeneratedUnionVariant5X3aclass9X3amainX2eBird1X3a0(Bird::new("Tweety"
        .to_string(), 0.3_f64).clone()))
    );
    let v: Option<SifrInt> = Some(SifrInt::from_i64(42));
    println!("{}", find_value(v.clone(), SifrInt::from_i64(42)));
    println!("{}", find_value(v.clone(), SifrInt::from_i64(99)));
    println!("{}", is_positive(v.clone()));
    let empty: Vec<String> = Vec::new();
    println!("{}", summarize(&empty));
    let full: Vec<String> = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    println!("{}", summarize(&full));
}
