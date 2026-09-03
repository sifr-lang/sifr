// src/main.rs
mod sifr_generated_project_unions {
    use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr {
        SifrGeneratedUnionVariant4X3aatom3X3aint(SifrInt),
        SifrGeneratedUnionVariant4X3aatom3X3astr(String),
    }
    impl ::std::fmt::Display
        for SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr
    {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr::SifrGeneratedUnionVariant4X3aatom3X3aint(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr::SifrGeneratedUnionVariant4X3aatom3X3astr(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
}
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr;
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
fn describe_number(x: SifrInt) -> String {
    match x {
        SifrInt::Small(0) => "zero".to_string(),
        SifrInt::Small(1) => "one".to_string(),
        SifrInt::Small(2) => "two".to_string(),
        _ => "many".to_string(),
    }
}
fn classify_http(method: &str) -> String {
    match method {
        sifr_generated_s if sifr_generated_s == "GET" || sifr_generated_s == "HEAD" => {
            "read".to_string()
        }
        sifr_generated_s
            if sifr_generated_s == "POST"
                || sifr_generated_s == "PUT"
                || sifr_generated_s == "PATCH" =>
        {
            "write".to_string()
        }
        sifr_generated_s if sifr_generated_s == "DELETE" => "delete".to_string(),
        _ => "other".to_string(),
    }
}
fn classify_score(score: SifrInt) -> String {
    match score {
        n if &n >= &SifrInt::from_i64(90) => "A".to_string(),
        n if &n >= &SifrInt::from_i64(80) => "B".to_string(),
        n if &n >= &SifrInt::from_i64(70) => "C".to_string(),
        n if &n >= &SifrInt::from_i64(60) => "D".to_string(),
        _ => "F".to_string(),
    }
}
fn describe_optional(x: Option<SifrInt>) -> String {
    if x.is_none() {
        "nothing".to_string()
    } else {
        "something".to_string()
    }
}
fn describe_union(
    x: &SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr,
) -> String {
    match x {
        SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr::SifrGeneratedUnionVariant4X3aatom3X3aint(
            ..,
        ) => "integer".to_string(),
        SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr::SifrGeneratedUnionVariant4X3aatom3X3astr(
            ..,
        ) => "string".to_string(),
    }
}
const fn make_int_union()
-> SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr {
    SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr::SifrGeneratedUnionVariant4X3aatom3X3aint(
        SifrInt::from_i64(42),
    )
}
fn make_str_union()
-> SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr {
    SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr::SifrGeneratedUnionVariant4X3aatom3X3astr(
        "hello".to_string(),
    )
}
fn classify_point(p: &Point) -> String {
    match p {
        Point {
            x: SifrInt::Small(0),
            y: SifrInt::Small(0),
            ..
        } => "origin".to_string(),
        Point {
            x: px,
            y: SifrInt::Small(0),
            ..
        } => {
            let _px = px.clone();
            "on x-axis".to_string()
        }
        Point {
            x: SifrInt::Small(0),
            y: py,
            ..
        } => {
            let _py = py.clone();
            "on y-axis".to_string()
        }
        Point { x: px, y: py, .. } => {
            let _px = px.clone();
            let _py = py.clone();
            "general".to_string()
        }
    }
}
fn classify_pair(p: (SifrInt, SifrInt)) -> String {
    match p {
        (SifrInt::Small(0), SifrInt::Small(0)) => "origin".to_string(),
        (_x, SifrInt::Small(0)) => "x-axis".to_string(),
        (SifrInt::Small(0), _y) => "y-axis".to_string(),
        (_x, _y) => "general".to_string(),
    }
}
fn classify_quadrant(p: &Point) -> String {
    match p {
        Point {
            x: SifrInt::Small(0),
            y: SifrInt::Small(0),
            ..
        } => "origin".to_string(),
        Point { x: px, y: py, .. }
            if &*px > &SifrInt::from_i64(0) && &*py > &SifrInt::from_i64(0) =>
        {
            let _px = px.clone();
            let _py = py.clone();
            "Q1".to_string()
        }
        Point { x: px, y: py, .. }
            if &*px < &SifrInt::from_i64(0) && &*py > &SifrInt::from_i64(0) =>
        {
            let _px = px.clone();
            let _py = py.clone();
            "Q2".to_string()
        }
        _ => "other".to_string(),
    }
}
fn main() {
    println!("=== Literal Patterns ===");
    println!("{}", describe_number(SifrInt::from_i64(0)));
    println!("{}", describe_number(SifrInt::from_i64(1)));
    println!("{}", describe_number(SifrInt::from_i64(42)));
    println!("=== OR Patterns ===");
    println!("{}", classify_http(&"GET".to_string()));
    println!("{}", classify_http(&"POST".to_string()));
    println!("{}", classify_http(&"DELETE".to_string()));
    println!("{}", classify_http(&"OPTIONS".to_string()));
    println!("=== Guard Patterns ===");
    println!("{}", classify_score(SifrInt::from_i64(95)));
    println!("{}", classify_score(SifrInt::from_i64(85)));
    println!("{}", classify_score(SifrInt::from_i64(55)));
    println!("=== Optional Matching ===");
    println!("{}", describe_optional(None));
    println!("{}", describe_optional(Some(SifrInt::from_i64(42))));
    println!("=== Union Matching ===");
    let a: SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr =
        make_int_union();
    let b: SifrGeneratedUnion8X3asequence5X3aunion1X3a211X3a4X3aatom3X3aint11X3a4X3aatom3X3astr =
        make_str_union();
    println!("{}", describe_union(&a));
    println!("{}", describe_union(&b));
    println!("=== Class Destructuring ===");
    let p1: Point = Point::new(SifrInt::from_i64(0), SifrInt::from_i64(0));
    let p2: Point = Point::new(SifrInt::from_i64(3), SifrInt::from_i64(0));
    let p3: Point = Point::new(SifrInt::from_i64(0), SifrInt::from_i64(4));
    let p4: Point = Point::new(SifrInt::from_i64(3), SifrInt::from_i64(4));
    println!("{}", classify_point(&p1));
    println!("{}", classify_point(&p2));
    println!("{}", classify_point(&p3));
    println!("{}", classify_point(&p4));
    println!("=== Tuple Patterns ===");
    let t1: (SifrInt, SifrInt) = (SifrInt::from_i64(0), SifrInt::from_i64(0));
    let t2: (SifrInt, SifrInt) = (SifrInt::from_i64(3), SifrInt::from_i64(0));
    let t3: (SifrInt, SifrInt) = (SifrInt::from_i64(0), SifrInt::from_i64(4));
    let t4: (SifrInt, SifrInt) = (SifrInt::from_i64(3), SifrInt::from_i64(4));
    println!("{}", classify_pair(t1.clone()));
    println!("{}", classify_pair(t2.clone()));
    println!("{}", classify_pair(t3.clone()));
    println!("{}", classify_pair(t4.clone()));
    println!("=== Nested Patterns ===");
    println!(
        "{}",
        classify_quadrant(&Point::new(SifrInt::from_i64(0), SifrInt::from_i64(0)))
    );
    println!(
        "{}",
        classify_quadrant(&Point::new(SifrInt::from_i64(3), SifrInt::from_i64(4)))
    );
    println!(
        "{}",
        classify_quadrant(&Point::new(-&SifrInt::from_i64(2), SifrInt::from_i64(5)))
    );
    println!(
        "{}",
        classify_quadrant(&Point::new(-&SifrInt::from_i64(1), -&SifrInt::from_i64(1)))
    );
}
