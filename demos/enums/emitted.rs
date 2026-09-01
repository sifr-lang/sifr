// src/main.rs
use ::sifr_runtime::SifrInt;
use ::std::collections::HashMap;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i64)]
enum Color {
    RED = 1,
    GREEN = 2,
    BLUE = 3,
}
impl ::std::fmt::Display for Color {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Color {
    fn name(&self) -> String {
        format!("{self:?}")
    }
    fn value(&self) -> SifrInt {
        SifrInt::from_i64(*self as i64)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i64)]
enum HttpStatus {
    OK = 200,
    NOT_FOUND = 404,
    INTERNAL_ERROR = 500,
}
impl ::std::fmt::Display for HttpStatus {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl HttpStatus {
    fn name(&self) -> String {
        format!("{self:?}")
    }
    fn value(&self) -> SifrInt {
        SifrInt::from_i64(*self as i64)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i64)]
enum Direction {
    NORTH = 1,
    SOUTH = 2,
    EAST = 3,
}
impl ::std::fmt::Display for Direction {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Direction {
    fn name(&self) -> String {
        format!("{self:?}")
    }
    fn value(&self) -> SifrInt {
        SifrInt::from_i64(*self as i64)
    }
    const fn is_vertical(&self) -> bool {
        match self {
            Direction::NORTH | Direction::SOUTH => true,
            Direction::EAST => false,
        }
    }
    fn opposite(&self) -> String {
        match self {
            Direction::NORTH => "SOUTH".to_string(),
            Direction::SOUTH => "NORTH".to_string(),
            Direction::EAST => "WEST".to_string(),
        }
    }
}
fn describe_color(c: Color) -> String {
    match c {
        Color::RED => "red".to_string(),
        Color::GREEN => "green".to_string(),
        Color::BLUE => "blue".to_string(),
    }
}
fn handle_status(s: HttpStatus) -> String {
    match s {
        HttpStatus::OK => "success".to_string(),
        HttpStatus::NOT_FOUND => "not found".to_string(),
        HttpStatus::INTERNAL_ERROR => "server error".to_string(),
    }
}
fn main() {
    println!("=== Basic Enum ===");
    let c: Color = Color::RED;
    println!("{c}");
    println!("{}", c.name());
    println!("{}", c.value());
    println!("=== Comparison ===");
    println!("{}", c == Color::RED);
    println!("{}", c == Color::BLUE);
    println!("=== Pattern Matching ===");
    println!("{}", describe_color(Color::GREEN));
    println!("{}", describe_color(Color::BLUE));
    println!("=== HTTP Status ===");
    let s: HttpStatus = HttpStatus::NOT_FOUND;
    println!("{s}");
    println!("{}", s.value());
    println!("{}", handle_status(HttpStatus::OK));
    println!("{}", handle_status(HttpStatus::INTERNAL_ERROR));
    println!("=== Enum Methods ===");
    let d: Direction = Direction::NORTH;
    println!("{}", d.is_vertical());
    println!("{}", Direction::EAST.is_vertical());
    println!("{}", d.opposite());
    println!("{}", Direction::EAST.opposite());
    println!("=== Enum as Dict Key ===");
    let mut scores: HashMap<Color, SifrInt> = HashMap::from([]);
    {
        let sifr_generated_assign_value = SifrInt::from_i64(10);
        {
            let sifr_generated_assign_key = Color::RED;
            scores.insert(sifr_generated_assign_key, sifr_generated_assign_value);
        }
    }
    {
        let sifr_generated_assign_value = SifrInt::from_i64(20);
        {
            let sifr_generated_assign_key = Color::GREEN;
            scores.insert(sifr_generated_assign_key, sifr_generated_assign_value);
        }
    }
    {
        let sifr_generated_assign_value = SifrInt::from_i64(30);
        {
            let sifr_generated_assign_key = Color::BLUE;
            scores.insert(sifr_generated_assign_key, sifr_generated_assign_value);
        }
    }
    let v: Option<SifrInt> = scores.get(&Color::GREEN).cloned();
    if let Some(v) = v.clone() {
        println!("{v}");
    }
}
