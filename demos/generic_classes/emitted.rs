// src/main.rs
use ::sifr_runtime::SifrInt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Pair<T> {
    first: T,
    second: T,
}

impl<T> Pair<T> {
    fn new(first: T, second: T) -> Self {
        let __sifr_field_init_0: T = first;
        let __sifr_field_init_1: T = second;
        Self { first: __sifr_field_init_0, second: __sifr_field_init_1 }
    }
}

impl<T: Clone> Pair<T> {
    fn swap(&self) -> Pair<T> {
        Pair::new(self.second.clone(), self.first.clone())
    }
}

impl<T: ::std::fmt::Display> ::std::fmt::Display for Pair<T> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Pair(first={}, second={})", self.first, self.second)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Stack<T> {
    items: Vec<T>,
}

impl<T> Stack<T> {
    fn new(items: Vec<T>) -> Self {
        let __sifr_field_init_0: Vec<T> = items;
        Self { items: __sifr_field_init_0 }
    }
}

impl<T: Clone> Stack<T> {
    fn push(&mut self, item: &T) {
        self.items.push(item.clone());
    }
}

impl<T> Stack<T> {
    fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }
}

impl<T> Stack<T> {
    fn size(&self) -> SifrInt {
        SifrInt::from(self.items.len())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Wrapper<T> {
    value: T,
}

impl<T> Wrapper<T> {
    fn new(value: T) -> Self {
        let __sifr_field_init_0: T = value;
        Self { value: __sifr_field_init_0 }
    }
}

impl<T: Clone> Wrapper<T> {
    fn get(&self) -> T {
        self.value.clone()
    }
}

impl<T: ::std::fmt::Display> ::std::fmt::Display for Wrapper<T> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Wrapper(value={})", self.value)
    }
}

fn main() {
    let p: Pair<SifrInt> = Pair::new(SifrInt::from_i64(10), SifrInt::from_i64(20));
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(13usize + 0usize);
    __sifr_concat.push_str("pair first = ");
    __sifr_concat.push_str(format!("{}", p.first.clone()).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(14usize + 0usize);
    __sifr_concat.push_str("pair second = ");
    __sifr_concat.push_str(format!("{}", p.second.clone()).as_str());
    __sifr_concat
});
    let p2: Pair<SifrInt> = p.swap();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(16usize + 0usize);
    __sifr_concat.push_str("swapped first = ");
    __sifr_concat.push_str(format!("{}", p2.first.clone()).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(17usize + 0usize);
    __sifr_concat.push_str("swapped second = ");
    __sifr_concat.push_str(format!("{}", p2.second.clone()).as_str());
    __sifr_concat
});
    let sp: Pair<String> = Pair::new("hello".to_string(), "world".to_string());
    let sp2: Pair<String> = sp.swap();
    println!("str pair swap ok = true");
    let mut s: Stack<SifrInt> = Stack::new(vec![]);
    s.push(&SifrInt::from_i64(1));
    s.push(&SifrInt::from_i64(2));
    s.push(&SifrInt::from_i64(3));
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(13usize + 0usize);
    __sifr_concat.push_str("stack size = ");
    __sifr_concat.push_str(format!("{}", s.size()).as_str());
    __sifr_concat
});
    let item: Option<SifrInt> = s.pop();
    if let Some(item) = item.clone() {
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(9usize + 0usize);
    __sifr_concat.push_str("popped = ");
    __sifr_concat.push_str(format!("{}", item).as_str());
    __sifr_concat
});
    }
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(23usize + 0usize);
    __sifr_concat.push_str("stack size after pop = ");
    __sifr_concat.push_str(format!("{}", s.size()).as_str());
    __sifr_concat
});
    let w: Wrapper<SifrInt> = Wrapper::new(SifrInt::from_i64(42));
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(14usize + 0usize);
    __sifr_concat.push_str("wrapper get = ");
    __sifr_concat.push_str(format!("{}", w.get()).as_str());
    __sifr_concat
});
    let x: () = ();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(12usize + 0usize);
    __sifr_concat.push_str("x is None = ");
    __sifr_concat.push_str(format!("{}", true).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(16usize + 0usize);
    __sifr_concat.push_str("x is not None = ");
    __sifr_concat.push_str(format!("{}", false).as_str());
    __sifr_concat
});
}
