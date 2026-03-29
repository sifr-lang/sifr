fn identity<T>(value: T) -> T {
    value
}

fn repeat<T: Clone>(value: T, n: usize) -> Vec<T> {
    std::iter::repeat(value).take(n).collect()
}

struct Container<T> {
    value: T,
}

impl<T> Container<T> {
    fn new(value: T) -> Self {
        Self { value }
    }

    fn get(self) -> T {
        self.value
    }
}

trait Printable {
    fn display(&self) -> String;
}

struct User {
    name: String,
}

impl Printable for User {
    fn display(&self) -> String {
        format!("User({})", self.name)
    }
}

struct Product {
    title: String,
    price: f64,
}

impl Printable for Product {
    fn display(&self) -> String {
        format!("Product({}, ${})", self.title, self.price)
    }
}

fn show(item: Box<dyn Printable>) {
    println!("{}", item.display());
}

fn main() {
    println!("=== PEP 695 Generic Functions ===");
    println!("{}", identity(42_i64));
    println!("{}", identity("hello"));
    println!("{:?}", repeat("x", 3));

    println!("=== PEP 695 Generic Classes ===");
    let c = Container::new(99_i64);
    println!("{}", c.get());
    let c2 = Container::new("wrapped");
    println!("{}", c2.get());

    println!("=== Protocol Method Dispatch ===");
    let user = User {
        name: "Alice".to_string(),
    };
    let product = Product {
        title: "Widget".to_string(),
        price: 9.99,
    };
    show(Box::new(user));
    show(Box::new(product));

    println!("=== Multi-Generator Comprehensions ===");
    let matrix = [[1_i64, 2, 3], [4, 5, 6], [7, 8, 9]];
    let flat: Vec<i64> = matrix.into_iter().flatten().collect();
    println!("{flat:?}");

    println!("=== Stdlib Math Functions ===");
    println!("{}", 1.0_f64.ln());
    println!("{}", 0.0_f64.sin());
    println!("{}", 0.0_f64.cos());
    println!("{}", (-42.0_f64).abs());
    println!("{}", 2.0_f64.powf(10.0));
    println!("{}", 3.14_f64.round() as i64);
}
