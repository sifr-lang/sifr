#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Timer {
    label: String,
}

impl Timer {
    fn new(label: String) -> Self {
        return Self { label: label };
    }
    fn __enter__(&self) -> Timer {
        return self.clone();
    }
    fn __exit__(&self) {
        return;
    }
}

impl std::fmt::Display for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Timer(label={})", self.label);
    }
}

trait Describable {
    fn describe(&self) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Item {
    name: String,
}

impl Item {
    fn new(name: String) -> Self {
        return Self { name: name };
    }
    fn describe(&self) -> String {
        return format!("{}{}", "Item: ".to_string(), self.name.clone());
    }
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Item(name={})", self.name);
    }
}

impl Describable for Item {
    fn describe(&self) -> String {
        return Item::describe(self);
    }
}

fn main() {
    let nums: Vec<i64> = vec![5 as i64, 3 as i64, 1 as i64, 4 as i64, 2 as i64];
    let lo: Option<i64> = (nums).iter().copied().min();
    let hi: Option<i64> = (nums).iter().copied().max();
    if let Some(lo) = lo {
        println!("{}", lo);
    }
    if let Some(hi) = hi {
        println!("{}", hi);
    }
    let evens: Vec<i64> = Box::new(nums.iter().copied().filter(|__filter_item| {
        let __filter_value = *__filter_item;
        return {
            let x = __filter_value;
            (x % (2 as i64)) == (0 as i64)
        };
    }))
    .collect::<Vec<_>>();
    println!("{:?}", evens);
    let big: Vec<i64> = {
        let mut __sifr_list_comp = vec![];
        for x in nums.iter().copied() {
            if x > (2 as i64) {
                __sifr_list_comp.push(x);
            }
        }
        __sifr_list_comp
    };
    println!("{:?}", big);
    let name: String = "World".to_string();
    let greeting: String = format!("{}{}{}", "Hello, ".to_string(), name, "!".to_string());
    println!("{}", greeting);
    {
        let mut __ctx_0 = Timer::new("work".to_string());
        struct __WithGuard0 {
            ctx: Timer,
        }
        impl Drop for __WithGuard0 {
            fn drop(&mut self) {
                self.ctx.__exit__();
            }
        }
        let mut __guard_0 = __WithGuard0 { ctx: __ctx_0 };
        let _t = __guard_0.ctx.__enter__();
        println!("doing work inside with block");
    }
    let mut item: Item = Item::new("Widget".to_string());
    println!("{}", item.describe());
    println!("All codegen quality v2 improvements verified!");
}
