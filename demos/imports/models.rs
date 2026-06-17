// Reference: import semantics work
// Reference: imports
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct User {
    pub name: String,
    pub email: String,
}

impl User {
    pub fn new(name: String, email: String) -> Self {
        Self {
            name: name,
            email: email,
        }
    }

    pub fn display(&self) -> String {
        return format!("{} <{}>", self.name, self.email);
    }

}

#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    pub name: String,
    pub price: f64,
}

impl Product {
    pub fn new(name: String, price: f64) -> Self {
        Self {
            name: name,
            price: price,
        }
    }

    pub fn label(&self) -> String {
        return format!("{}: ${}", self.name, self.price);
    }

}

