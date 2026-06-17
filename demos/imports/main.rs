// Reference: import semantics work
// Reference: imports
mod utils;
mod models;

use crate::models::User;
use crate::models::Product;
use crate::utils::greet;
use crate::utils::format_total;

fn main() {
    let mut msg: String = greet("Alice".to_string());
    println!("{}", msg);
    let mut user: User = User::new("Alice".to_string(), "alice@example.com".to_string());
    println!("{}", user.display());
    let mut product: Product = Product::new("Widget".to_string(), 9.99_f64);
    println!("{}", product.label());
    let mut summary: String = format_total(3_i64, 29.97_f64);
    println!("{}", summary);
    println!("{}", "multi-file compilation works!".to_string());
}
