// src/main.rs
use ::sifr_runtime::SifrInt;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Pair<T> {
    first: T,
    second: T,
}
impl<T> Pair<T> {
    const fn new(first: T, second: T) -> Self {
        let sifr_generated_field_value_89d7ed7f996f1d41_6669727374: T = first;
        let sifr_generated_field_value_a49985ef4cee20bd_7365636f6e64: T = second;
        Self {
            first: sifr_generated_field_value_89d7ed7f996f1d41_6669727374,
            second: sifr_generated_field_value_a49985ef4cee20bd_7365636f6e64,
        }
    }
}
impl<T: Clone> Pair<T> {
    fn swap(&self) -> Self {
        Self::new(self.second.clone(), self.first.clone())
    }
}
impl<T: ::std::fmt::Display> ::std::fmt::Display for Pair<T> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Pair(first={}, second={})", self.first, self.second)
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct Stack<T> {
    items: Vec<T>,
}
impl<T> Stack<T> {
    const fn new(items: Vec<T>) -> Self {
        let sifr_generated_field_value_3e7884bf4f412c6f_6974656d73: Vec<T> = items;
        Self {
            items: sifr_generated_field_value_3e7884bf4f412c6f_6974656d73,
        }
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
    const fn new(value: T) -> Self {
        let sifr_generated_field_value_7ce4fd9430e80cea_76616c7565: T = value;
        Self {
            value: sifr_generated_field_value_7ce4fd9430e80cea_76616c7565,
        }
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
        let mut sifr_generated_concat: String =
            String::with_capacity(13usize.saturating_add(0usize));
        sifr_generated_concat.push_str("pair first = ");
        sifr_generated_concat.push_str(p.first.to_string().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(14usize.saturating_add(0usize));
        sifr_generated_concat.push_str("pair second = ");
        sifr_generated_concat.push_str(p.second.to_string().as_str());
        sifr_generated_concat
    });
    let p2: Pair<SifrInt> = p.swap();
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(16usize.saturating_add(0usize));
        sifr_generated_concat.push_str("swapped first = ");
        sifr_generated_concat.push_str(p2.first.to_string().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(17usize.saturating_add(0usize));
        sifr_generated_concat.push_str("swapped second = ");
        sifr_generated_concat.push_str(p2.second.to_string().as_str());
        sifr_generated_concat
    });
    let sp: Pair<String> = Pair::new("hello".to_string(), "world".to_string());
    let _sp2: Pair<String> = sp.swap();
    println!("str pair swap ok = true");
    let mut s: Stack<SifrInt> = Stack::new(Vec::new());
    s.push(&SifrInt::from_i64(1));
    s.push(&SifrInt::from_i64(2));
    s.push(&SifrInt::from_i64(3));
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(13usize.saturating_add(0usize));
        sifr_generated_concat.push_str("stack size = ");
        sifr_generated_concat.push_str(s.size().to_string().as_str());
        sifr_generated_concat
    });
    let item: Option<SifrInt> = s.pop();
    if let Some(item) = item {
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(9usize.saturating_add(0usize));
            sifr_generated_concat.push_str("popped = ");
            sifr_generated_concat.push_str(item.to_string().as_str());
            sifr_generated_concat
        });
    }
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(23usize.saturating_add(0usize));
        sifr_generated_concat.push_str("stack size after pop = ");
        sifr_generated_concat.push_str(s.size().to_string().as_str());
        sifr_generated_concat
    });
    let w: Wrapper<SifrInt> = Wrapper::new(SifrInt::from_i64(42));
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(14usize.saturating_add(0usize));
        sifr_generated_concat.push_str("wrapper get = ");
        sifr_generated_concat.push_str(w.get().to_string().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(12usize.saturating_add(0usize));
        sifr_generated_concat.push_str("x is None = ");
        sifr_generated_concat.push_str(true.to_string().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(16usize.saturating_add(0usize));
        sifr_generated_concat.push_str("x is not None = ");
        sifr_generated_concat.push_str(false.to_string().as_str());
        sifr_generated_concat
    });
}
