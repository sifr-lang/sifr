// src/main.rs
mod models;

use crate::models::User;

use crate::models::Product;

fn main() {
    let user: User = User::new("Alice".to_string(), "alice@example.com".to_string());
    println!("{}", user.display());
    let product: Product = Product::new("Widget".to_string(), 9.99_f64);
    println!("{}", product.label());
    println!("multi-file compilation works!");
}

// src/models.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct User {
    pub name: String,
    pub email: String,
}
impl User {
    pub fn new(name: String, email: String) -> Self {
        let __sifr_field_init_0: String = name;
        let __sifr_field_init_1: String = email;
        Self {
            name: __sifr_field_init_0,
            email: __sifr_field_init_1,
        }
    }
}
impl User {
    pub fn display(&self) -> String {
        format!("{} <{}>", self.name.clone(), self.email.clone())
    }
}
impl ::std::fmt::Display for User {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "User(name={}, email={})", self.name, self.email)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub name: String,
    pub price: f64,
}
impl Product {
    pub fn new(name: String, price: f64) -> Self {
        let __sifr_field_init_0: String = name;
        let __sifr_field_init_1: f64 = price;
        Self {
            name: __sifr_field_init_0,
            price: __sifr_field_init_1,
        }
    }
}
impl Product {
    pub fn label(&self) -> String {
        format!("{}: ${}", self.name.clone(), self.price)
    }
}
impl ::std::fmt::Display for Product {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Product(name={}, price={})", self.name, self.price)
    }
}
