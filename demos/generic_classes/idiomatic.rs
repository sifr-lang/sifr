#[derive(Debug, Clone, PartialEq)]
struct Pair<T: Clone + std::fmt::Display + PartialOrd> {
    first: T,
    second: T,
}

impl<T: Clone + std::fmt::Display + PartialOrd> Pair<T> {
    fn new(first: T, second: T) -> Self {
        return Self {
            first: first,
            second: second,
        };
    }
    fn swap(&self) -> Pair<T> {
        return Pair::new(self.second.clone(), self.first.clone());
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Stack<T: Clone + std::fmt::Display + PartialOrd> {
    items: Vec<T>,
}

impl<T: Clone + std::fmt::Display + PartialOrd> Stack<T> {
    fn new(items: Vec<T>) -> Self {
        return Self { items: items };
    }
    fn push(&mut self, item: &T) {
        self.items.push(item.clone());
    }
    fn pop(&mut self) -> Option<T> {
        return self.items.pop();
    }
    fn size(&self) -> i64 {
        return self.items.clone().len() as i64;
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Wrapper<T: Clone + std::fmt::Display + PartialOrd> {
    value: T,
}

impl<T: Clone + std::fmt::Display + PartialOrd> Wrapper<T> {
    fn new(value: T) -> Self {
        return Self { value: value };
    }
    fn get(&self) -> T {
        return self.value.clone();
    }
}

fn main() {
    let mut p = Pair::new(10 as i64, 20 as i64);
    println!(
        "{}",
        format!("{}{}", "pair first = ".to_string(), format!("{}", p.first))
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "pair second = ".to_string(),
            format!("{}", p.second)
        )
    );
    let p2 = p.swap();
    println!(
        "{}",
        format!(
            "{}{}",
            "swapped first = ".to_string(),
            format!("{}", p2.first)
        )
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "swapped second = ".to_string(),
            format!("{}", p2.second)
        )
    );
    let mut sp = Pair::new("hello".to_string(), "world".to_string());
    let sp2 = sp.swap();
    println!("str pair swap ok = true");
    let mut s = Stack::new(vec![]);
    s.push(&(1 as i64));
    s.push(&(2 as i64));
    s.push(&(3 as i64));
    println!(
        "{}",
        format!("{}{}", "stack size = ".to_string(), format!("{}", s.size()))
    );
    let item: Option<i64> = s.pop();
    if let Some(item) = item {
        println!(
            "{}",
            format!("{}{}", "popped = ".to_string(), format!("{}", item))
        );
    }
    println!(
        "{}",
        format!(
            "{}{}",
            "stack size after pop = ".to_string(),
            format!("{}", s.size())
        )
    );
    let mut w = Wrapper::new(42 as i64);
    println!(
        "{}",
        format!("{}{}", "wrapper get = ".to_string(), format!("{}", w.get()))
    );
    let x: () = ();
    println!(
        "{}",
        format!("{}{}", "x is None = ".to_string(), format!("{}", true))
    );
    println!(
        "{}",
        format!("{}{}", "x is not None = ".to_string(), format!("{}", false))
    );
}
