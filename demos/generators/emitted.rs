// src/main.rs
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

fn fibonacci(n: i64) -> Box<dyn Iterator<Item = i64>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<i64> = Vec::new().into_iter();
    Box::new(::std::iter::from_fn(move || {
    if !__sifr_generator_initialized {
        let mut _yields: Vec<i64> = Vec::new();
        let mut a: i64 = 0_i64;
        let mut b: i64 = 1_i64;
        let mut i: i64 = 0_i64;
        while i < n {
            _yields.push(a);
            let temp: i64 = a + b;
            a = b;
            b = temp;
            i += 1_i64;
        }
        __sifr_generator_iter = _yields.into_iter();
        __sifr_generator_initialized = true;
    }
    __sifr_generator_iter.next()
}))
}

fn evens(limit: i64) -> Box<dyn Iterator<Item = i64>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<i64> = Vec::new().into_iter();
    Box::new(::std::iter::from_fn(move || {
    if !__sifr_generator_initialized {
        let mut _yields: Vec<i64> = Vec::new();
        let mut i: i64 = 0_i64;
        while i < limit {
            if (i % (2_i64)) == (0_i64) {
                _yields.push(i);
            }
            i += 1_i64;
        }
        __sifr_generator_iter = _yields.into_iter();
        __sifr_generator_initialized = true;
    }
    __sifr_generator_iter.next()
}))
}

fn main() {
    let fibs: Vec<i64> = fibonacci(8_i64).collect::<Vec<_>>();
    println!("{:?}", fibs);
    let even_nums: Vec<i64> = evens(10_i64).collect::<Vec<_>>();
    println!("{:?}", even_nums);
    let nums: Vec<i64> = vec![1_i64, 2_i64, 3_i64, 4_i64, 5_i64];
    for x in nums.iter().copied() {
        println!("{}", x * x);
    }
    {
        let mut __ctx_0 = Timer::new("work".to_string());
        struct __WithGuard0 { ctx: Timer }
        impl Drop for __WithGuard0 {
            fn drop(&mut self) { self.ctx.__exit__(); }
        }
        let mut __guard_0 = __WithGuard0 { ctx: __ctx_0 };
        let _t = __guard_0.ctx.__enter__();
        println!("doing work");
    }
    println!("done");
}
