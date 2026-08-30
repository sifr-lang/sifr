// src/main.rs
use ::sifr_runtime::SifrInt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Entity {
    id: SifrInt,
}

impl Entity {
    fn new(id: SifrInt) -> Self {
        let __sifr_field_init_0: SifrInt = id.clone();
        Self { id: __sifr_field_init_0 }
    }
}

impl Entity {
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
    fn new(id: SifrInt, name: String) -> Self {
        let __sifr_parent = Entity::new(id);
        let __sifr_field_init_0: String = name;
        Self { entity: __sifr_parent, name: __sifr_field_init_0 }
    }
}

impl Person {
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
    fn new(id: SifrInt, name: String, level: SifrInt) -> Self {
        let __sifr_parent = Person::new(id, name);
        let __sifr_field_init_0: SifrInt = level.clone();
        Self { person: __sifr_parent, level: __sifr_field_init_0 }
    }
}

impl Employee {
}

impl ::std::fmt::Display for Employee {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Employee(person={}, level={})", self.person, self.level)
    }
}

fn sum_items(values: &Vec<SifrInt>) -> SifrInt {
    let mut total: SifrInt = SifrInt::from_i64(0);
    for value in values.iter().cloned() {
        total = &total + &value;
    }
    total.clone()
}

fn main() {
    println!("variance_rules inheritance and variance corrections demo:");
    let emp: Employee = Employee::new(SifrInt::from_i64(11), "Lin".to_string(), SifrInt::from_i64(4));
    println!("{}", emp.person.name.clone());
    println!("{}", sum_items(&vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)]));
}
