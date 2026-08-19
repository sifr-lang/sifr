// src/main.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Point {
    x: i64,
    y: i64,
}

impl Point {
    fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }
}

impl Point {
}

impl ::std::fmt::Display for Point {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Point(x={}, y={})", self.x, self.y)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Config {
    debug: bool,
    timeout: i64,
    name: String,
}

impl Config {
    fn new(debug: bool, timeout: i64, name: String) -> Self {
        Self { debug, timeout, name }
    }
}

impl Config {
}

impl ::std::fmt::Display for Config {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Config(debug={}, timeout={}, name={})", self.debug, self.timeout, self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Person {
    first_name: String,
    last_name: String,
    age: i64,
}

impl Person {
    fn new(first_name: String, last_name: String, age: i64) -> Self {
        Self { first_name, last_name, age }
    }
}

impl Person {
}

impl ::std::fmt::Display for Person {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Person(first_name={}, last_name={}, age={})", self.first_name, self.last_name, self.age)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Rectangle {
    width: i64,
    height: i64,
}

impl Rectangle {
    fn new(width: i64, height: i64) -> Self {
        let __sifr_field_init_0: i64 = width;
        let __sifr_field_init_1: i64 = height;
        Self { width: __sifr_field_init_0, height: __sifr_field_init_1 }
    }
}

impl Rectangle {
    fn area(&self) -> i64 {
        self.width * self.height
    }
}

impl ::std::fmt::Display for Rectangle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", {
    let mut __sifr_concat: String = String::with_capacity((((10usize + 0usize) + 1usize) + 0usize) + 1usize);
    __sifr_concat.push_str("Rectangle(");
    __sifr_concat.push_str((format!("{}", self.width)).as_str());
    __sifr_concat.push('x');
    __sifr_concat.push_str((format!("{}", self.height)).as_str());
    __sifr_concat.push(')');
    __sifr_concat
})
    }
}

fn main() {
    let p: Point = Point::new(3_i64, 4_i64);
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(10usize + 0usize);
    __sifr_concat.push_str("point x = ");
    __sifr_concat.push_str((format!("{}", p.x)).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(10usize + 0usize);
    __sifr_concat.push_str("point y = ");
    __sifr_concat.push_str((format!("{}", p.y)).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(12usize + 0usize);
    __sifr_concat.push_str("point str = ");
    __sifr_concat.push_str((format!("{}", p)).as_str());
    __sifr_concat
});
    let p2: Point = Point::new(3_i64, 4_i64);
    let p3: Point = Point::new(5_i64, 6_i64);
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(11usize + 0usize);
    __sifr_concat.push_str("point eq = ");
    __sifr_concat.push_str((format!("{}", p == p2)).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(12usize + 0usize);
    __sifr_concat.push_str("point neq = ");
    __sifr_concat.push_str((format!("{}", p == p3)).as_str());
    __sifr_concat
});
    let c1: Config = Config::new(false, 30_i64, "default".to_string());
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(23usize + 0usize);
    __sifr_concat.push_str("config debug default = ");
    __sifr_concat.push_str((format!("{}", c1.debug)).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(25usize + 0usize);
    __sifr_concat.push_str("config timeout default = ");
    __sifr_concat.push_str((format!("{}", c1.timeout)).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(22usize + 0usize);
    __sifr_concat.push_str("config name default = ");
    __sifr_concat.push_str((c1.name.clone()).as_str());
    __sifr_concat
});
    let c2: Config = Config::new(true, 60_i64, "production".to_string());
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(22usize + 0usize);
    __sifr_concat.push_str("config debug custom = ");
    __sifr_concat.push_str((format!("{}", c2.debug)).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(24usize + 0usize);
    __sifr_concat.push_str("config timeout custom = ");
    __sifr_concat.push_str((format!("{}", c2.timeout)).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(21usize + 0usize);
    __sifr_concat.push_str("config name custom = ");
    __sifr_concat.push_str((c2.name.clone()).as_str());
    __sifr_concat
});
    let person: Person = Person::new("Alice".to_string(), "Smith".to_string(), 30_i64);
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(13usize + 0usize);
    __sifr_concat.push_str("person str = ");
    __sifr_concat.push_str((format!("{}", person)).as_str());
    __sifr_concat
});
    let r: Rectangle = Rectangle::new(5_i64, 3_i64);
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(12usize + 0usize);
    __sifr_concat.push_str("rect area = ");
    __sifr_concat.push_str((format!("{}", r.area())).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(11usize + 0usize);
    __sifr_concat.push_str("rect str = ");
    __sifr_concat.push_str((format!("{}", r)).as_str());
    __sifr_concat
});
}
