// src/main.rs
use ::sifr_runtime::SifrInt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Timer {
    label: String,
}

impl Timer {
    fn new(label: String) -> Self {
        let __sifr_field_init_0: String = label;
        Self { label: __sifr_field_init_0 }
    }
}

impl Timer {
    fn __enter__(&self) -> Timer {
        self.clone()
    }
}

impl Timer {
    fn __exit__(&self) {
    }
}

impl ::std::fmt::Display for Timer {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Timer(label={})", self.label)
    }
}

pub trait Describable {
    fn describe(&self) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Item {
    name: String,
}

impl Item {
    fn new(name: String) -> Self {
        let __sifr_field_init_0: String = name;
        Self { name: __sifr_field_init_0 }
    }
}

impl Item {
    fn describe(&self) -> String {
        {
    let mut __sifr_concat: String = String::with_capacity(6usize + 0usize);
    __sifr_concat.push_str("Item: ");
    __sifr_concat.push_str((self.name.clone()).as_str());
    __sifr_concat
}
    }
}

impl ::std::fmt::Display for Item {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Item(name={})", self.name)
    }
}

impl Describable for Item {
    fn describe(&self) -> String {
        Item::describe(self)
    }
}

fn main() {
    let nums: Vec<SifrInt> = vec![SifrInt::from_i64(5), SifrInt::from_i64(3), SifrInt::from_i64(1), SifrInt::from_i64(4), SifrInt::from_i64(2)];
    let lo: Option<SifrInt> = (nums).iter().cloned().min();
    let hi: Option<SifrInt> = (nums).iter().cloned().max();
    if let Some(lo) = lo.clone() {
        println!("{}", lo);
    }
    if let Some(hi) = hi.clone() {
        println!("{}", hi);
    }
    let evens: Vec<SifrInt> = Box::new((nums).iter().cloned().filter(move |__filter_item| {
    let x = __filter_item.clone();
    (&x.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0))
})).collect::<Vec<_>>();
    println!("{:?}", evens);
    let big: Vec<SifrInt> = {
    let mut __sifr_list_comp = vec![];
    for x in nums.iter().cloned() {
        if &x > &SifrInt::from_i64(2) {
            __sifr_list_comp.push(x);
        }
    }
    __sifr_list_comp
};
    println!("{:?}", big);
    let name: String = "World".to_string();
    let greeting: String = {
    let mut __sifr_concat: String = String::with_capacity((7usize + name.len()) + 1usize);
    __sifr_concat.push_str("Hello, ");
    __sifr_concat.push_str((name).as_str());
    __sifr_concat.push('!');
    __sifr_concat
};
    println!("{}", greeting);
    {
        let mut __ctx_0 = Timer::new("work".to_string());
        struct __WithGuard0 { ctx: Timer }
        impl Drop for __WithGuard0 {
            fn drop(&mut self) { self.ctx.__exit__(); }
        }
        let mut __guard_0 = __WithGuard0 { ctx: __ctx_0 };
        let _t = __guard_0.ctx.__enter__();
        println!("doing work inside with block");
    }
    let item: Item = Item::new("Widget".to_string());
    println!("{}", item.describe());
    println!("All codegen quality v2 improvements verified!");
}
