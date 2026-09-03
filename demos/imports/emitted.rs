// src/main.rs
pub mod models;
use crate::models::Product;
use crate::models::User;
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
    #[must_use]
    pub const fn new(name: String, email: String) -> Self {
        let sifr_generated_field_value_c4bcadba8e631b86_6e616d65: String = name;
        let sifr_generated_field_value_123467b419acbc07_656d61696c: String = email;
        Self {
            name: sifr_generated_field_value_c4bcadba8e631b86_6e616d65,
            email: sifr_generated_field_value_123467b419acbc07_656d61696c,
        }
    }
}
impl User {
    #[must_use]
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
    #[must_use]
    pub const fn new(name: String, price: f64) -> Self {
        let sifr_generated_field_value_c4bcadba8e631b86_6e616d65: String = name;
        let sifr_generated_field_value_2f1887248c8bc0ea_7072696365: f64 = price;
        Self {
            name: sifr_generated_field_value_c4bcadba8e631b86_6e616d65,
            price: sifr_generated_field_value_2f1887248c8bc0ea_7072696365,
        }
    }
}
impl Product {
    #[must_use]
    pub fn label(&self) -> String {
        format!("{}: ${}", self.name.clone(), self.price)
    }
}
impl ::std::fmt::Display for Product {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Product(name={}, price={})", self.name, self.price)
    }
}
