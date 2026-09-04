// src/main.rs
use ::sifr_runtime::SifrInt;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Entity {
    id: SifrInt,
}
impl Entity {
    const fn new(id: SifrInt) -> Self {
        let sifr_generated_field_value_08b72e07b55c3ac0_6964: SifrInt = id;
        Self {
            id: sifr_generated_field_value_08b72e07b55c3ac0_6964,
        }
    }
}
impl ::std::fmt::Display for Entity {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Entity(id={})", self.id)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Person {
    entity: Entity,
    name: String,
}
impl ::std::ops::Deref for Person {
    type Target = Entity;
    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}
impl ::std::ops::DerefMut for Person {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entity
    }
}
impl ::std::convert::From<Person> for Entity {
    fn from(value: Person) -> Self {
        value.entity
    }
}
impl Person {
    const fn new(id: SifrInt, name: String) -> Self {
        let sifr_generated_parent = Entity::new(id);
        let sifr_generated_field_value_c4bcadba8e631b86_6e616d65: String = name;
        Self {
            entity: sifr_generated_parent,
            name: sifr_generated_field_value_c4bcadba8e631b86_6e616d65,
        }
    }
}
impl ::std::fmt::Display for Person {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Person(entity={}, name={})", self.entity, self.name)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Employee {
    person: Person,
    level: SifrInt,
}
impl ::std::ops::Deref for Employee {
    type Target = Person;
    fn deref(&self) -> &Self::Target {
        &self.person
    }
}
impl ::std::ops::DerefMut for Employee {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.person
    }
}
impl ::std::convert::From<Employee> for Person {
    fn from(value: Employee) -> Self {
        value.person
    }
}
impl Employee {
    const fn new(id: SifrInt, name: String, level: SifrInt) -> Self {
        let sifr_generated_parent = Person::new(id, name);
        let sifr_generated_field_value_e8ddc90a9d7c709d_6c6576656c: SifrInt = level;
        Self {
            person: sifr_generated_parent,
            level: sifr_generated_field_value_e8ddc90a9d7c709d_6c6576656c,
        }
    }
}
impl ::std::fmt::Display for Employee {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Employee(person={}, level={})", self.person, self.level)
    }
}
fn sum_items(values: &[SifrInt]) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    #[expect(
        clippy::explicit_iter_loop,
        reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
    )]
    for value in values.iter() {
        total = ::std::ops::Add::add(&total, value);
    }
    total
}
fn main() {
    println!("variance_rules inheritance and variance corrections demo:");
    let emp: Employee = Employee::new(
        SifrInt::from_i64(11),
        "Lin".to_string(),
        SifrInt::from_i64(4),
    );
    println!("{}", emp.person.name);
    println!(
        "{}",
        sum_items(&[
            SifrInt::from_i64(1),
            SifrInt::from_i64(2),
            SifrInt::from_i64(3)
        ])
    );
}
