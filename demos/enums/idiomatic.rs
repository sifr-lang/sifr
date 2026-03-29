use std::collections::HashMap;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i64)]
enum Color {
    Red = 1,
    Green = 2,
    Blue = 3,
}

impl Color {
    fn name(self) -> &'static str {
        match self {
            Self::Red => "RED",
            Self::Green => "GREEN",
            Self::Blue => "BLUE",
        }
    }

    fn value(self) -> i64 {
        self as i64
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i64)]
enum HttpStatus {
    Ok = 200,
    NotFound = 404,
    InternalError = 500,
}

impl HttpStatus {
    fn name(self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::NotFound => "NOT_FOUND",
            Self::InternalError => "INTERNAL_ERROR",
        }
    }

    fn value(self) -> i64 {
        self as i64
    }
}

impl fmt::Display for HttpStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i64)]
#[allow(dead_code)]
enum Direction {
    North = 1,
    South = 2,
    East = 3,
    West = 4,
}

impl Direction {
    fn is_vertical(self) -> bool {
        matches!(self, Self::North | Self::South)
    }

    fn opposite(self) -> &'static str {
        match self {
            Self::North => "SOUTH",
            Self::South => "NORTH",
            Self::East => "WEST",
            Self::West => "EAST",
        }
    }
}

fn describe_color(color: Color) -> &'static str {
    match color {
        Color::Red => "red",
        Color::Green => "green",
        Color::Blue => "blue",
    }
}

fn handle_status(status: HttpStatus) -> &'static str {
    match status {
        HttpStatus::Ok => "success",
        HttpStatus::NotFound => "not found",
        HttpStatus::InternalError => "server error",
    }
}

fn main() {
    println!("=== Basic Enum ===");
    let color = Color::Red;
    println!("{color}");
    println!("{}", color.name());
    println!("{}", color.value());

    println!("=== Comparison ===");
    println!("{}", color == Color::Red);
    println!("{}", color == Color::Blue);

    println!("=== Pattern Matching ===");
    println!("{}", describe_color(Color::Green));
    println!("{}", describe_color(Color::Blue));

    println!("=== HTTP Status ===");
    let status = HttpStatus::NotFound;
    println!("{status}");
    println!("{}", status.value());
    println!("{}", handle_status(HttpStatus::Ok));
    println!("{}", handle_status(HttpStatus::InternalError));

    println!("=== Enum Methods ===");
    let direction = Direction::North;
    println!("{}", direction.is_vertical());
    println!("{}", Direction::East.is_vertical());
    println!("{}", direction.opposite());
    println!("{}", Direction::East.opposite());

    println!("=== Enum as Dict Key ===");
    let scores = HashMap::from([(Color::Red, 10), (Color::Green, 20), (Color::Blue, 30)]);
    if let Some(score) = scores.get(&Color::Green) {
        println!("{score}");
    }
}
