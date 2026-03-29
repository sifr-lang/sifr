fn factorial(n: i64) -> i64 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

fn fibonacci(n: i64) -> i64 {
    match n {
        i64::MIN..=-1 => 0,
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn greet(name: &str) -> String {
    format!("Hello, {name}!")
}

fn classify(x: i64) -> &'static str {
    if x > 0 {
        "positive"
    } else if x < 0 {
        "negative"
    } else {
        "zero"
    }
}

fn double(n: i64) -> i64 {
    n * 2
}

fn is_even(n: i64) -> bool {
    match n {
        0 => true,
        1 => false,
        _ => is_even(n - 2),
    }
}

fn main() {
    let x = 42;
    let _pi = 3.14;
    let _flag = true;
    let name = "Sifr";

    println!("{}", double(x + 8));
    println!("{}", factorial(5));
    println!("{}", fibonacci(10));
    println!("{}", greet(name));
    println!("{}", classify(x));
    println!("{}", classify(-7));
    println!("{}", classify(0));
    println!("{}", is_even(4));
    println!("{}", is_even(7));
}
