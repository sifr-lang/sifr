// src/main.rs
mod sifr_generated_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ::std::fmt::Display for ValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ValueError {}
}
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::ValueError;
fn log(x: f64) -> f64 {
    ::sifr_stdlib::math::log(x)
}
fn sin(x: f64) -> f64 {
    ::sifr_stdlib::math::sin(x)
}
fn cos(x: f64) -> f64 {
    ::sifr_stdlib::math::cos(x)
}
fn fabs(x: f64) -> f64 {
    ::sifr_stdlib::math::fabs(x)
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Container<T> {
    value: T,
}
impl<T> Container<T> {
    const fn new(value: T) -> Self {
        let sifr_generated_field_value_7ce4fd9430e80cea_76616c7565: T = value;
        Self {
            value: sifr_generated_field_value_7ce4fd9430e80cea_76616c7565,
        }
    }
}
impl<T: Clone> Container<T> {
    fn get(&self) -> T {
        self.value.clone()
    }
}
impl<T: ::std::fmt::Display> ::std::fmt::Display for Container<T> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Container(value={})", self.value)
    }
}
pub trait Printable {
    fn display(&self) -> String;
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct User {
    name: String,
}
impl User {
    const fn new(name: String) -> Self {
        let sifr_generated_field_value_c4bcadba8e631b86_6e616d65: String = name;
        Self {
            name: sifr_generated_field_value_c4bcadba8e631b86_6e616d65,
        }
    }
}
impl User {
    fn display(&self) -> String {
        format!("User({})", self.name.clone())
    }
}
impl ::std::fmt::Display for User {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "User(name={})", self.name)
    }
}
impl Printable for User {
    fn display(&self) -> String {
        User::display(self)
    }
}
#[derive(Debug, Clone, PartialEq)]
struct Product {
    title: String,
    price: f64,
}
impl Product {
    const fn new(title: String, price: f64) -> Self {
        let sifr_generated_field_value_da31296c0c1b6029_7469746c65: String = title;
        let sifr_generated_field_value_2f1887248c8bc0ea_7072696365: f64 = price;
        Self {
            title: sifr_generated_field_value_da31296c0c1b6029_7469746c65,
            price: sifr_generated_field_value_2f1887248c8bc0ea_7072696365,
        }
    }
}
impl Product {
    fn display(&self) -> String {
        format!("Product({}, ${})", self.title.clone(), self.price)
    }
}
impl ::std::fmt::Display for Product {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Product(title={}, price={})", self.title, self.price)
    }
}
impl Printable for Product {
    fn display(&self) -> String {
        Product::display(self)
    }
}
fn identity<T: Clone + 'static>(x: &T) -> T {
    x.clone()
}
fn repeat<T: Clone + 'static>(x: &T, n: SifrInt) -> Vec<T> {
    let mut result: Vec<T> = Vec::new();
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &n {
        result.push(x.clone());
        i = &i + &SifrInt::from_i64(1);
    }
    result
}
fn show(item: Box<dyn Printable>) {
    println!("{}", item.display());
}
#[expect(
    clippy::approx_constant,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
fn main() {
    println!("=== PEP 695 Generic Functions ===");
    println!("{}", identity(&SifrInt::from_i64(42)));
    println!("{}", identity(&"hello".to_string()));
    println!("{:?}", repeat(&"x".to_string(), SifrInt::from_i64(3)));
    println!("=== PEP 695 Generic Classes ===");
    let c: Container<SifrInt> = Container::new(SifrInt::from_i64(99));
    println!("{}", c.get());
    let c2: Container<String> = Container::new("wrapped".to_string());
    println!("{}", c2.get());
    println!("=== Protocol Method Dispatch ===");
    let u: User = User::new("Alice".to_string());
    let pr: Product = Product::new("Widget".to_string(), 9.99_f64);
    show(Box::new(u));
    show(Box::new(pr));
    println!("=== Multi-Generator Comprehensions ===");
    let matrix: Vec<Vec<SifrInt>> = vec![
        vec![
            SifrInt::from_i64(1),
            SifrInt::from_i64(2),
            SifrInt::from_i64(3),
        ],
        vec![
            SifrInt::from_i64(4),
            SifrInt::from_i64(5),
            SifrInt::from_i64(6),
        ],
        vec![
            SifrInt::from_i64(7),
            SifrInt::from_i64(8),
            SifrInt::from_i64(9),
        ],
    ];
    let flat: Vec<SifrInt> = {
        let mut sifr_generated_list_comp = Vec::new();
        for row in matrix.iter().cloned() {
            for x in row.iter().cloned() {
                sifr_generated_list_comp.push(x);
            }
        }
        sifr_generated_list_comp
    };
    println!("{flat:?}");
    println!("=== Stdlib Math Functions ===");
    println!("{}", log(1.0_f64));
    println!("{}", sin(0.0_f64));
    println!("{}", cos(0.0_f64));
    println!("{}", fabs(-42.0_f64));
    println!("{}", (2.0_f64 as f64).powf(10.0_f64 as f64));
    println!(
        "{:?}",
        SifrInt::from_f64_trunc(3.14_f64.round_ties_even()).ok_or_else(|| ValueError {
            message: "cannot round non-finite float to int".to_string()
        })
    );
}
