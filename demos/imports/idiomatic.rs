mod models {
    pub struct User {
        name: String,
        email: String,
    }

    impl User {
        pub fn new(name: &str, email: &str) -> Self {
            Self {
                name: name.to_string(),
                email: email.to_string(),
            }
        }

        pub fn display(&self) -> String {
            format!("{} <{}>", self.name, self.email)
        }
    }

    pub struct Product {
        name: String,
        price: f64,
    }

    impl Product {
        pub fn new(name: &str, price: f64) -> Self {
            Self {
                name: name.to_string(),
                price,
            }
        }

        pub fn label(&self) -> String {
            format!("{}: ${}", self.name, self.price)
        }
    }
}

use models::{Product, User};

fn main() {
    let user = User::new("Alice", "alice@example.com");
    println!("{}", user.display());

    let product = Product::new("Widget", 9.99);
    println!("{}", product.label());

    println!("multi-file compilation works!");
}
