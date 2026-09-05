// src/main.rs
use ::sifr_runtime::SifrInt;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Point {
    x: SifrInt,
    y: SifrInt,
}
impl Point {
    const fn new(x: SifrInt, y: SifrInt) -> Self {
        Self { x, y }
    }
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
    const fn new(debug: bool, timeout: SifrInt, name: String) -> Self {
        Self {
            debug,
            timeout,
            name,
        }
    }
}
impl ::std::fmt::Display for Config {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(
            f,
            "Config(debug={}, timeout={}, name={})",
            self.debug, self.timeout, self.name
        )
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Person {
    first_name: String,
    last_name: String,
    age: SifrInt,
}
impl Person {
    const fn new(first_name: String, last_name: String, age: SifrInt) -> Self {
        Self {
            first_name,
            last_name,
            age,
        }
    }
}
impl ::std::fmt::Display for Person {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(
            f,
            "Person(first_name={}, last_name={}, age={})",
            self.first_name, self.last_name, self.age
        )
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Rectangle {
    width: SifrInt,
    height: SifrInt,
}
impl Rectangle {
    fn new(width: &SifrInt, height: &SifrInt) -> Self {
        let sifr_generated_field_value_dbdacd932fd1e9bf_7769647468: SifrInt = (*width).clone();
        let sifr_generated_field_value_17720bf67d347222_686569676874: SifrInt = (*height).clone();
        Self {
            width: sifr_generated_field_value_dbdacd932fd1e9bf_7769647468,
            height: sifr_generated_field_value_17720bf67d347222_686569676874,
        }
    }
}
impl Rectangle {
    fn area(&self) -> SifrInt {
        ::std::ops::Mul::mul(&self.width.clone(), &self.height.clone())
    }
}
impl ::std::fmt::Display for Rectangle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", {
            let mut sifr_generated_concat: String = String::with_capacity(
                10usize
                    .saturating_add(0usize)
                    .saturating_add(1usize)
                    .saturating_add(0usize)
                    .saturating_add(1usize),
            );
            sifr_generated_concat.push_str("Rectangle(");
            sifr_generated_concat.push_str(self.width.to_string().as_str());
            sifr_generated_concat.push('x');
            sifr_generated_concat.push_str(self.height.to_string().as_str());
            sifr_generated_concat.push(')');
            sifr_generated_concat
        })
    }
}
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    let p: Point = Point::new(SifrInt::from_i64(3), SifrInt::from_i64(4));
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(10usize.saturating_add(0usize));
        sifr_generated_concat.push_str("point x = ");
        sifr_generated_concat.push_str(p.x.to_string().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(10usize.saturating_add(0usize));
        sifr_generated_concat.push_str("point y = ");
        sifr_generated_concat.push_str(p.y.to_string().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(12usize.saturating_add(0usize));
        sifr_generated_concat.push_str("point str = ");
        sifr_generated_concat.push_str(p.to_string().as_str());
        sifr_generated_concat
    });
    let p2: Point = Point::new(SifrInt::from_i64(3), SifrInt::from_i64(4));
    let p3: Point = Point::new(SifrInt::from_i64(5), SifrInt::from_i64(6));
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(11usize.saturating_add(0usize));
        sifr_generated_concat.push_str("point eq = ");
        sifr_generated_concat.push_str((p == p2).to_string().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(12usize.saturating_add(0usize));
        sifr_generated_concat.push_str("point neq = ");
        sifr_generated_concat.push_str((p == p3).to_string().as_str());
        sifr_generated_concat
    });
    let c1: Config = Config::new(false, SifrInt::from_i64(30), "default".to_string());
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(23usize.saturating_add(0usize));
        sifr_generated_concat.push_str("config debug default = ");
        sifr_generated_concat.push_str(c1.debug.to_string().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(25usize.saturating_add(0usize));
        sifr_generated_concat.push_str("config timeout default = ");
        sifr_generated_concat.push_str(c1.timeout.to_string().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(22usize.saturating_add(0usize));
        sifr_generated_concat.push_str("config name default = ");
        sifr_generated_concat.push_str(c1.name.as_str());
        sifr_generated_concat
    });
    let c2: Config = Config::new(true, SifrInt::from_i64(60), "production".to_string());
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(22usize.saturating_add(0usize));
        sifr_generated_concat.push_str("config debug custom = ");
        sifr_generated_concat.push_str(c2.debug.to_string().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(24usize.saturating_add(0usize));
        sifr_generated_concat.push_str("config timeout custom = ");
        sifr_generated_concat.push_str(c2.timeout.to_string().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(21usize.saturating_add(0usize));
        sifr_generated_concat.push_str("config name custom = ");
        sifr_generated_concat.push_str(c2.name.as_str());
        sifr_generated_concat
    });
    let person: Person = Person::new(
        "Alice".to_string(),
        "Smith".to_string(),
        SifrInt::from_i64(30),
    );
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(13usize.saturating_add(0usize));
        sifr_generated_concat.push_str("person str = ");
        sifr_generated_concat.push_str(person.to_string().as_str());
        sifr_generated_concat
    });
    let r: Rectangle = Rectangle::new(&SifrInt::from_i64(5), &SifrInt::from_i64(3));
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(12usize.saturating_add(0usize));
        sifr_generated_concat.push_str("rect area = ");
        sifr_generated_concat.push_str(r.area().to_string().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(11usize.saturating_add(0usize));
        sifr_generated_concat.push_str("rect str = ");
        sifr_generated_concat.push_str(r.to_string().as_str());
        sifr_generated_concat
    });
}
