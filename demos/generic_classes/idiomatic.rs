#[derive(Debug, Clone, PartialEq)]
struct Pair<T> {
    first: T,
    second: T,
}

impl<T> Pair<T> {
    fn new(first: T, second: T) -> Self {
        Self { first, second }
    }
}

impl<T: Clone> Pair<T> {
    fn swap(&self) -> Self {
        Self::new(self.second.clone(), self.first.clone())
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    fn new(items: Vec<T>) -> Self {
        Self { items }
    }

    fn push(&mut self, item: T) {
        self.items.push(item);
    }

    fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    fn size(&self) -> i64 {
        self.items.len() as i64
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Wrapper<T> {
    value: T,
}

impl<T> Wrapper<T> {
    fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T: Clone> Wrapper<T> {
    fn get(&self) -> T {
        self.value.clone()
    }
}

fn main() {
    let pair = Pair::new(10_i64, 20_i64);
    println!("pair first = {}", pair.first);
    println!("pair second = {}", pair.second);

    let swapped = pair.swap();
    println!("swapped first = {}", swapped.first);
    println!("swapped second = {}", swapped.second);

    let string_pair = Pair::new("hello".to_string(), "world".to_string());
    let _ = string_pair.swap();
    println!("str pair swap ok = true");

    let mut stack = Stack::new(Vec::new());
    stack.push(1_i64);
    stack.push(2_i64);
    stack.push(3_i64);
    println!("stack size = {}", stack.size());
    if let Some(item) = stack.pop() {
        println!("popped = {}", item);
    }
    println!("stack size after pop = {}", stack.size());

    let wrapper = Wrapper::new(42_i64);
    println!("wrapper get = {}", wrapper.get());

    let x: Option<()> = None;
    println!("x is None = {}", x.is_none());
    println!("x is not None = {}", x.is_some());
}
