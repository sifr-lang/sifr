// src/main.rs
use ::sifr_runtime::SifrInt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Point {
    x: SifrInt,
    y: SifrInt,
}

impl Point {
    fn new(x: SifrInt, y: SifrInt) -> Self {
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
    timeout: SifrInt,
    name: String,
}

impl Config {
    fn new(debug: bool, timeout: SifrInt, name: String) -> Self {
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
    age: SifrInt,
}

impl Person {
    fn new(first_name: String, last_name: String, age: SifrInt) -> Self {
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
    width: SifrInt,
    height: SifrInt,
}

impl Rectangle {
    fn new(width: SifrInt, height: SifrInt) -> Self {
        let __sifr_field_init_0: SifrInt = width.clone();
        let __sifr_field_init_1: SifrInt = height.clone();
        Self { width: __sifr_field_init_0, height: __sifr_field_init_1 }
    }
}

impl Rectangle {
    fn area(&self) -> SifrInt {
        &self.width.clone() * &self.height.clone()
    }
}

impl ::std::fmt::Display for Rectangle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", {
    let mut __sifr_concat: String = String::with_capacity((((10usize + 0usize) + 1usize) + 0usize) + 1usize);
    __sifr_concat.push_str("Rectangle(");
    __sifr_concat.push_str(format!("{}", self.width.clone()).as_str());
    __sifr_concat.push('x');
    __sifr_concat.push_str(format!("{}", self.height.clone()).as_str());
    __sifr_concat.push(')');
    __sifr_concat
})
    }
}

fn main() {
    let p: Point = Point::new(SifrInt::from_i64(3), SifrInt::from_i64(4));
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(10usize + 0usize);
    __sifr_concat.push_str("point x = ");
    __sifr_concat.push_str(format!("{}", p.x.clone()).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(10usize + 0usize);
    __sifr_concat.push_str("point y = ");
    __sifr_concat.push_str(format!("{}", p.y.clone()).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(12usize + 0usize);
    __sifr_concat.push_str("point str = ");
    __sifr_concat.push_str(format!("{}", p).as_str());
    __sifr_concat
});
    let p2: Point = Point::new(SifrInt::from_i64(3), SifrInt::from_i64(4));
    let p3: Point = Point::new(SifrInt::from_i64(5), SifrInt::from_i64(6));
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(11usize + 0usize);
    __sifr_concat.push_str("point eq = ");
    __sifr_concat.push_str(format!("{}", p == p2).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(12usize + 0usize);
    __sifr_concat.push_str("point neq = ");
    __sifr_concat.push_str(format!("{}", p == p3).as_str());
    __sifr_concat
});
    let c1: Config = Config::new(false, SifrInt::from_i64(30), "default".to_string());
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(23usize + 0usize);
    __sifr_concat.push_str("config debug default = ");
    __sifr_concat.push_str(format!("{}", c1.debug).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(25usize + 0usize);
    __sifr_concat.push_str("config timeout default = ");
    __sifr_concat.push_str(format!("{}", c1.timeout.clone()).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(22usize + 0usize);
    __sifr_concat.push_str("config name default = ");
    __sifr_concat.push_str(c1.name.clone().as_str());
    __sifr_concat
});
    let c2: Config = Config::new(true, SifrInt::from_i64(60), "production".to_string());
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(22usize + 0usize);
    __sifr_concat.push_str("config debug custom = ");
    __sifr_concat.push_str(format!("{}", c2.debug).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(24usize + 0usize);
    __sifr_concat.push_str("config timeout custom = ");
    __sifr_concat.push_str(format!("{}", c2.timeout.clone()).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(21usize + 0usize);
    __sifr_concat.push_str("config name custom = ");
    __sifr_concat.push_str(c2.name.clone().as_str());
    __sifr_concat
});
    let person: Person = Person::new("Alice".to_string(), "Smith".to_string(), SifrInt::from_i64(30));
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(13usize + 0usize);
    __sifr_concat.push_str("person str = ");
    __sifr_concat.push_str(format!("{}", person).as_str());
    __sifr_concat
});
    let r: Rectangle = Rectangle::new(SifrInt::from_i64(5), SifrInt::from_i64(3));
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(12usize + 0usize);
    __sifr_concat.push_str("rect area = ");
    __sifr_concat.push_str(format!("{}", r.area()).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(11usize + 0usize);
    __sifr_concat.push_str("rect str = ");
    __sifr_concat.push_str(format!("{}", r).as_str());
    __sifr_concat
});
}
