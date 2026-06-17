#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Entity {
    id: i64,
}

impl Entity {
    fn new(id: i64) -> Self {
        return Self { id: id };
    }
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Entity(id={})", self.id);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Person {
    entity: Entity,
    name: String,
}

impl Person {
    fn new(id: i64, name: String) -> Self {
        return Self { entity: Entity::new(id), name: name };
    }
}

impl std::fmt::Display for Person {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Person(name={})", self.name);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Employee {
    person: Person,
    level: i64,
}

impl Employee {
    fn new(id: i64, name: String, level: i64) -> Self {
        return Self { person: Person::new(id, name), level: level };
    }
}

impl std::fmt::Display for Employee {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Employee(level={})", self.level);
    }
}

fn sum_items(values: &Vec<i64>) -> i64 {
    let mut total: i64 = 0 as i64;
    for value in values.iter().copied() {
        total = total + value;
    }
    return total;
}

fn main() {
    println!("variance_rules inheritance and variance corrections demo:");
    let emp: Employee = Employee::new(11 as i64, "Lin".to_string(), 4 as i64);
    println!("{}", emp.person.name);
    println!("{}", sum_items(&vec![1 as i64, 2 as i64, 3 as i64]));
}
