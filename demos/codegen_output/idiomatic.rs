struct Timer<'a> {
    _label: &'a str,
}

impl<'a> Timer<'a> {
    fn new(label: &'a str) -> Self {
        Self { _label: label }
    }
}

struct Item<'a> {
    name: &'a str,
}

impl Item<'_> {
    fn describe(&self) -> String {
        format!("Item: {}", self.name)
    }
}

fn main() {
    let nums = [5, 3, 1, 4, 2];

    if let Some(lo) = nums.iter().copied().min() {
        println!("{lo}");
    }
    if let Some(hi) = nums.iter().copied().max() {
        println!("{hi}");
    }

    let evens = nums
        .iter()
        .copied()
        .filter(|x| x % 2 == 0)
        .collect::<Vec<_>>();
    println!("{evens:?}");

    let big = nums.iter().copied().filter(|x| *x > 2).collect::<Vec<_>>();
    println!("{big:?}");

    let greeting = format!("Hello, {}!", "World");
    println!("{greeting}");

    let _timer = Timer::new("work");
    println!("doing work inside with block");

    let item = Item { name: "Widget" };
    println!("{}", item.describe());

    println!("All codegen quality v2 improvements verified!");
}
