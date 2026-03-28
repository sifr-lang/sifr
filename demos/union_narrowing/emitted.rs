#[derive(Debug, Clone)]
enum BirdOrCatOrDog {
    Bird(Bird),
    Cat(Cat),
    Dog(Dog),
}

impl std::fmt::Display for BirdOrCatOrDog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BirdOrCatOrDog::Bird(v) => {
                return write!(f, "{:?}", v);
            },
            BirdOrCatOrDog::Cat(v) => {
                return write!(f, "{:?}", v);
            },
            BirdOrCatOrDog::Dog(v) => {
                return write!(f, "{:?}", v);
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Dog {
    name: String,
    breed: String,
}

impl Dog {
    fn new(name: String, breed: String) -> Self {
        return Self { name: name, breed: breed };
    }
}

impl std::fmt::Display for Dog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Dog(name={}, breed={})", self.name, self.breed);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Cat {
    name: String,
    color: String,
}

impl Cat {
    fn new(name: String, color: String) -> Self {
        return Self { name: name, color: color };
    }
}

impl std::fmt::Display for Cat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Cat(name={}, color={})", self.name, self.color);
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Bird {
    name: String,
    wingspan: f64,
}

impl Bird {
    fn new(name: String, wingspan: f64) -> Self {
        return Self { name: name, wingspan: wingspan };
    }
}

impl std::fmt::Display for Bird {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Bird(name={}, wingspan={})", self.name, self.wingspan);
    }
}

fn route_handler(method: &String) -> String {
    if method.clone() == "GET".to_string() {
        return "get handler".to_string();
    } else {
        if method.clone() == "POST".to_string() {
            return "post handler".to_string();
        } else {
            if method.clone() == "PUT".to_string() {
                return "put handler".to_string();
            }
        }
    }
    return "unknown".to_string();
}

fn describe_pet(pet: &BirdOrCatOrDog) -> String {
    if let BirdOrCatOrDog::Dog(pet) = pet {
        return format!("{} is a {}", pet.name, pet.breed);
    } else {
        if let BirdOrCatOrDog::Cat(pet) = pet {
            return format!("{} is {}", pet.name, pet.color);
        } else {
            if let BirdOrCatOrDog::Bird(pet) = pet {
                return format!("{} has wingspan {}", pet.name, pet.wingspan);
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
    return "not found".to_string();
}

fn is_positive(x: Option<i64>) -> bool {
    if x > Some(0 as i64) {
        return true;
    }
    return false;
}

fn summarize(items: &Vec<String>) -> String {
    if items.is_empty() {
        return "no items".to_string();
    }
    return format!("{} items", items.len() as i64);
}

fn main() {
    println!("{}", route_handler(&"GET".to_string()));
    println!("{}", route_handler(&"POST".to_string()));
    println!("{}", route_handler(&"PUT".to_string()));
    println!("{}", route_handler(&"DELETE".to_string()));
    println!("{}", describe_pet(&BirdOrCatOrDog::Dog(Dog::new("Rex".to_string(), "Labrador".to_string()))));
    println!("{}", describe_pet(&BirdOrCatOrDog::Cat(Cat::new("Whiskers".to_string(), "orange".to_string()))));
    println!("{}", describe_pet(&BirdOrCatOrDog::Bird(Bird::new("Tweety".to_string(), 0.3 as f64))));
    let v: Option<i64> = Some(42 as i64);
    println!("{}", find_value(v, 42 as i64));
    println!("{}", find_value(v, 99 as i64));
    println!("{}", is_positive(v));
    let empty: Vec<String> = vec![];
    println!("{}", summarize(&empty));
    let full: Vec<String> = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    println!("{}", summarize(&full));
}
