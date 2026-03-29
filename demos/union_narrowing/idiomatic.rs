#[derive(Debug, Clone)]
enum BirdOrCatOrDog {
    Bird(Bird),
    Cat(Cat),
    Dog(Dog),
}

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

fn route_handler(method: &str) -> &'static str {
    match method {
        "GET" => "get handler",
        "POST" => "post handler",
        "PUT" => "put handler",
        _ => "unknown",
    }
}

fn describe_pet(pet: &BirdOrCatOrDog) -> String {
    match pet {
        BirdOrCatOrDog::Dog(pet) => format!("{} is a {}", pet.name, pet.breed),
        BirdOrCatOrDog::Cat(pet) => format!("{} is {}", pet.name, pet.color),
        BirdOrCatOrDog::Bird(pet) => format!("{} has wingspan {}", pet.name, pet.wingspan),
    }
}

fn find_value(x: Option<i64>, target: i64) -> String {
    if x == Some(target) {
        "found".to_string()
    } else {
        "not found".to_string()
    }
}

fn is_positive(x: Option<i64>) -> bool {
    x.is_some_and(|x| x > 0)
}

fn summarize(items: &[String]) -> String {
    if items.is_empty() {
        "no items".to_string()
    } else {
        format!("{} items", items.len())
    }
}

fn main() {
    println!("{}", route_handler("GET"));
    println!("{}", route_handler("POST"));
    println!("{}", route_handler("PUT"));
    println!("{}", route_handler("DELETE"));
    println!(
        "{}",
        describe_pet(&BirdOrCatOrDog::Dog(Dog::new(
            "Rex".to_string(),
            "Labrador".to_string()
        )))
    );
    println!(
        "{}",
        describe_pet(&BirdOrCatOrDog::Cat(Cat::new(
            "Whiskers".to_string(),
            "orange".to_string()
        )))
    );
    println!(
        "{}",
        describe_pet(&BirdOrCatOrDog::Bird(Bird::new(
            "Tweety".to_string(),
            0.3 as f64
        )))
    );
    let v = Some(42);
    println!("{}", find_value(v, 42));
    println!("{}", find_value(v, 99));
    println!("{}", is_positive(v));
    let empty = Vec::<String>::new();
    println!("{}", summarize(&empty));
    let full = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    println!("{}", summarize(&full));
}
