// src/main.rs
mod __sifr_project_unions {
    #[derive(Debug, Clone, PartialEq)]
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a320_x3a5_x3aclass8_x3amain_x2eCat1_x3a020_x3a5_x3aclass8_x3amain_x2eDog1_x3a021_x3a5_x3aclass9_x3amain_x2eBird1_x3a0 {
        __SifrUnionVariant_5_x3aclass9_x3amain_x2eBird1_x3a0(crate::Bird),
        __SifrUnionVariant_5_x3aclass8_x3amain_x2eCat1_x3a0(crate::Cat),
        __SifrUnionVariant_5_x3aclass8_x3amain_x2eDog1_x3a0(crate::Dog),
    }
    impl ::std::fmt::Display
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a320_x3a5_x3aclass8_x3amain_x2eCat1_x3a020_x3a5_x3aclass8_x3amain_x2eDog1_x3a021_x3a5_x3aclass9_x3amain_x2eBird1_x3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a320_x3a5_x3aclass8_x3amain_x2eCat1_x3a020_x3a5_x3aclass8_x3amain_x2eDog1_x3a021_x3a5_x3aclass9_x3amain_x2eBird1_x3a0::__SifrUnionVariant_5_x3aclass9_x3amain_x2eBird1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a320_x3a5_x3aclass8_x3amain_x2eCat1_x3a020_x3a5_x3aclass8_x3amain_x2eDog1_x3a021_x3a5_x3aclass9_x3amain_x2eBird1_x3a0::__SifrUnionVariant_5_x3aclass8_x3amain_x2eCat1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a320_x3a5_x3aclass8_x3amain_x2eCat1_x3a020_x3a5_x3aclass8_x3amain_x2eDog1_x3a021_x3a5_x3aclass9_x3amain_x2eBird1_x3a0::__SifrUnionVariant_5_x3aclass8_x3amain_x2eDog1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
            }
        }
    }
}
pub use __sifr_project_unions::__SifrUnion_8_x3asequence5_x3aunion1_x3a320_x3a5_x3aclass8_x3amain_x2eCat1_x3a020_x3a5_x3aclass8_x3amain_x2eDog1_x3a021_x3a5_x3aclass9_x3amain_x2eBird1_x3a0;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Dog {
    name: String,
    breed: String,
}

impl Dog {
    fn new(name: String, breed: String) -> Self {
        Self { name, breed }
    }
}

impl Dog {
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
    fn new(name: String, color: String) -> Self {
        Self { name, color }
    }
}

impl Cat {
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
    fn new(name: String, wingspan: f64) -> Self {
        Self { name, wingspan }
    }
}

impl Bird {
}

impl ::std::fmt::Display for Bird {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Bird(name={}, wingspan={})", self.name, self.wingspan)
    }
}

fn route_handler(method: &String) -> String {
    if (method).as_str() == "GET" {
        return "get handler".to_string();
    } else {
        if (method).as_str() == "POST" {
            return "post handler".to_string();
        } else {
            if (method).as_str() == "PUT" {
                return "put handler".to_string();
            }
        }
    }
    "unknown".to_string()
}

fn describe_pet(pet: &__SifrUnion_8_x3asequence5_x3aunion1_x3a320_x3a5_x3aclass8_x3amain_x2eCat1_x3a020_x3a5_x3aclass8_x3amain_x2eDog1_x3a021_x3a5_x3aclass9_x3amain_x2eBird1_x3a0) -> String {
    if let __SifrUnion_8_x3asequence5_x3aunion1_x3a320_x3a5_x3aclass8_x3amain_x2eCat1_x3a020_x3a5_x3aclass8_x3amain_x2eDog1_x3a021_x3a5_x3aclass9_x3amain_x2eBird1_x3a0::__SifrUnionVariant_5_x3aclass8_x3amain_x2eDog1_x3a0(pet) = pet {
        return format!("{} is a {}", pet.name.clone(), pet.breed.clone());
    } else {
        if let __SifrUnion_8_x3asequence5_x3aunion1_x3a320_x3a5_x3aclass8_x3amain_x2eCat1_x3a020_x3a5_x3aclass8_x3amain_x2eDog1_x3a021_x3a5_x3aclass9_x3amain_x2eBird1_x3a0::__SifrUnionVariant_5_x3aclass8_x3amain_x2eCat1_x3a0(pet) = pet {
            return format!("{} is {}", pet.name.clone(), pet.color.clone());
        } else {
            if let __SifrUnion_8_x3asequence5_x3aunion1_x3a320_x3a5_x3aclass8_x3amain_x2eCat1_x3a020_x3a5_x3aclass8_x3amain_x2eDog1_x3a021_x3a5_x3aclass9_x3amain_x2eBird1_x3a0::__SifrUnionVariant_5_x3aclass9_x3amain_x2eBird1_x3a0(pet) = pet {
                return format!("{} has wingspan {}", pet.name.clone(), pet.wingspan);
            } else {
                unreachable!("sifr union narrowing fell through exhaustive branch chain");
            }
        }
    }
}

fn find_value(x: Option<i64>, target: i64) -> String {
    if x == Some(target) {
        return "found".to_string();
    }
    "not found".to_string()
}

fn is_positive(x: Option<i64>) -> bool {
    if x > Some(0_i64) {
        return true;
    }
    false
}

fn summarize(items: &Vec<String>) -> String {
    if items.is_empty() {
        return "no items".to_string();
    }
    format!("{} items", items.len() as i64)
}

fn main() {
    println!("{}", route_handler(&"GET".to_string()));
    println!("{}", route_handler(&"POST".to_string()));
    println!("{}", route_handler(&"PUT".to_string()));
    println!("{}", route_handler(&"DELETE".to_string()));
    println!("{}", describe_pet(&__SifrUnion_8_x3asequence5_x3aunion1_x3a320_x3a5_x3aclass8_x3amain_x2eCat1_x3a020_x3a5_x3aclass8_x3amain_x2eDog1_x3a021_x3a5_x3aclass9_x3amain_x2eBird1_x3a0::__SifrUnionVariant_5_x3aclass8_x3amain_x2eDog1_x3a0((Dog::new("Rex".to_string(), "Labrador".to_string())).clone())));
    println!("{}", describe_pet(&__SifrUnion_8_x3asequence5_x3aunion1_x3a320_x3a5_x3aclass8_x3amain_x2eCat1_x3a020_x3a5_x3aclass8_x3amain_x2eDog1_x3a021_x3a5_x3aclass9_x3amain_x2eBird1_x3a0::__SifrUnionVariant_5_x3aclass8_x3amain_x2eCat1_x3a0((Cat::new("Whiskers".to_string(), "orange".to_string())).clone())));
    println!("{}", describe_pet(&__SifrUnion_8_x3asequence5_x3aunion1_x3a320_x3a5_x3aclass8_x3amain_x2eCat1_x3a020_x3a5_x3aclass8_x3amain_x2eDog1_x3a021_x3a5_x3aclass9_x3amain_x2eBird1_x3a0::__SifrUnionVariant_5_x3aclass9_x3amain_x2eBird1_x3a0((Bird::new("Tweety".to_string(), 0.3_f64)).clone())));
    let v: Option<i64> = Some(42_i64);
    println!("{}", find_value(v, 42_i64));
    println!("{}", find_value(v, 99_i64));
    println!("{}", is_positive(v));
    let empty: Vec<String> = vec![];
    println!("{}", summarize(&empty));
    let full: Vec<String> = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    println!("{}", summarize(&full));
}
