#[derive(Clone)]
struct ValueError(String);

#[derive(Clone)]
struct DivisionError(String);

#[derive(Clone)]
struct ParseError(String);

#[derive(Clone)]
struct AppError(String);

#[derive(Clone)]
struct Error(String);

fn validate_age(age: i64) -> Result<i64, ValueError> {
    if age < 0 {
        return Err(ValueError("age must be positive".to_string()));
    }
    if age > 150 {
        return Err(ValueError("too large".to_string()));
    }
    Ok(age)
}

fn safe_divide(a: i64, b: i64) -> Result<i64, DivisionError> {
    if b == 0 {
        return Err(DivisionError("division by zero".to_string()));
    }
    Ok(a / b)
}

fn check_input(value: i64) -> Result<i64, AppError> {
    if value < 0 {
        return Err(AppError("invalid input".to_string()));
    }
    Ok(value)
}

fn process_age(age: i64) -> Result<i64, ValueError> {
    validate_age(age)
}

fn parse_int(text: &str) -> Result<i64, ParseError> {
    text.parse::<i64>()
        .map_err(|error| ParseError(error.to_string()))
}

fn main() {
    println!("=== Built-in Error Classes ===");
    if let Err(error) = validate_age(-5) {
        println!("caught ValueError: {}", error.0);
    }
    if let Err(error) = safe_divide(10, 0) {
        println!("caught DivisionError: {}", error.0);
    }
    if let Err(error) = parse_int("not_a_number") {
        println!("caught ParseError: {}", error.0);
    }

    println!("=== Custom Error Classes ===");
    if let Err(error) = check_input(-1) {
        println!("caught AppError: {}", error.0);
    }

    println!("=== Exhaustiveness: Specific Except Arms ===");
    if let Err(error) = validate_age(-10) {
        println!("caught ValueError: {}", error.0);
    }

    println!("=== Exhaustiveness: Catch-All ===");
    if let Err(error) = validate_age(200).map_err(|error| Error(error.0)) {
        println!("caught: {}", error.0);
    }

    println!("=== Error Propagation ===");
    if let Err(error) = process_age(-1) {
        println!("pipeline error: {}", error.0);
    }

    println!("=== Multiple Try/Except ===");
    match parse_int("42") {
        Ok(parsed) => println!("parsed: {}", parsed),
        Err(error) => println!("parse error: {}", error.0),
    }
    match validate_age(42) {
        Ok(validated) => println!("validated: {}", validated),
        Err(error) => println!("validation error: {}", error.0),
    }
    match safe_divide(42, 6) {
        Ok(result) => println!("result: {}", result),
        Err(error) => println!("division error: {}", error.0),
    }

    println!("demo complete!");
}
