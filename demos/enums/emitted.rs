// src/main.rs
use ::sifr_runtime::SifrInt;
use ::std::collections::HashMap;
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i64)]
enum Color {
    Red = 1,
    Green = 2,
    Blue = 3,
}
impl ::std::fmt::Debug for Color {
    fn fmt(&self, formatter: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        formatter.write_str(match self {
            Self::Red => "RED",
            Self::Green => "GREEN",
            Self::Blue => "BLUE",
        })
    }
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
    const fn value(&self) -> SifrInt {
        SifrInt::from_i64(*self as i64)
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i64)]
enum HttpStatus {
    Ok = 200,
    NotFound = 404,
    InternalError = 500,
}
impl ::std::fmt::Debug for HttpStatus {
    fn fmt(&self, formatter: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        formatter.write_str(match self {
            Self::Ok => "OK",
            Self::NotFound => "NOT_FOUND",
            Self::InternalError => "INTERNAL_ERROR",
        })
    }
}
impl ::std::fmt::Display for HttpStatus {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl HttpStatus {
    const fn value(&self) -> SifrInt {
        SifrInt::from_i64(*self as i64)
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i64)]
enum Direction {
    North = 1,
    East = 3,
}
impl ::std::fmt::Debug for Direction {
    fn fmt(&self, formatter: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        formatter.write_str(match self {
            Self::North => "NORTH",
            Self::East => "EAST",
        })
    }
}
impl ::std::fmt::Display for Direction {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Direction {
    const fn is_vertical(&self) -> bool {
        match self {
            Direction::North => true,
            Direction::East => false,
        }
    }
    fn opposite(&self) -> String {
        match self {
            Direction::North => "SOUTH".to_string(),
            Direction::East => "WEST".to_string(),
        }
    }
}
fn describe_color(c: Color) -> String {
    match c {
        Color::Red => "red".to_string(),
        Color::Green => "green".to_string(),
        Color::Blue => "blue".to_string(),
    }
}
fn handle_status(s: HttpStatus) -> String {
    match s {
        HttpStatus::Ok => "success".to_string(),
        HttpStatus::NotFound => "not found".to_string(),
        HttpStatus::InternalError => "server error".to_string(),
    }
}
fn main() {
    println!("=== Basic Enum ===");
    let c: Color = Color::Red;
    println!("{c}");
    println!("{}", c.name());
    println!("{}", c.value());
    println!("=== Comparison ===");
    println!("{}", c == Color::Red);
    println!("{}", c == Color::Blue);
    println!("=== Pattern Matching ===");
    println!("{}", describe_color(Color::Green));
    println!("{}", describe_color(Color::Blue));
    println!("=== HTTP Status ===");
    let s: HttpStatus = HttpStatus::NotFound;
    println!("{s}");
    println!("{}", s.value());
    println!("{}", handle_status(HttpStatus::Ok));
    println!("{}", handle_status(HttpStatus::InternalError));
    println!("=== Enum Methods ===");
    let d: Direction = Direction::North;
    println!("{}", d.is_vertical());
    println!("{}", Direction::East.is_vertical());
    println!("{}", d.opposite());
    println!("{}", Direction::East.opposite());
    println!("=== Enum as Dict Key ===");
    let mut scores: HashMap<Color, SifrInt> = HashMap::from([]);
    {
        let sifr_generated_assign_value = SifrInt::from_i64(10);
        {
            let sifr_generated_assign_key = Color::Red;
            scores.insert(sifr_generated_assign_key, sifr_generated_assign_value);
        }
    }
    {
        let sifr_generated_assign_value = SifrInt::from_i64(20);
        {
            let sifr_generated_assign_key = Color::Green;
            scores.insert(sifr_generated_assign_key, sifr_generated_assign_value);
        }
    }
    {
        let sifr_generated_assign_value = SifrInt::from_i64(30);
        {
            let sifr_generated_assign_key = Color::Blue;
            scores.insert(sifr_generated_assign_key, sifr_generated_assign_value);
        }
    }
    let v: Option<SifrInt> = scores.get(&Color::Green).cloned();
    if let Some(v) = v.clone() {
        println!("{v}");
    }
}
