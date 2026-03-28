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

fn fibonacci(n: i64) -> Box<dyn Iterator<Item = i64>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: std::vec::IntoIter<i64> = Vec::new().into_iter();
    return Box::new(std::iter::from_fn(move || {
    if !__sifr_generator_initialized {
        let mut _yields: Vec<i64> = Vec::new();
        let mut a: i64 = 0 as i64;
        let mut b: i64 = 1 as i64;
        let mut i: i64 = 0 as i64;
        while i < n {
            _yields.push(a);
            let temp: i64 = a + b;
            a = b;
            b = temp;
            i = i + (1 as i64);
        }
        __sifr_generator_iter = _yields.into_iter();
        __sifr_generator_initialized = true;
    }
    return __sifr_generator_iter.next();
}));
}

fn evens(limit: i64) -> Box<dyn Iterator<Item = i64>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: std::vec::IntoIter<i64> = Vec::new().into_iter();
    return Box::new(std::iter::from_fn(move || {
    if !__sifr_generator_initialized {
        let mut _yields: Vec<i64> = Vec::new();
        let mut i: i64 = 0 as i64;
        while i < limit {
            if (i % (2 as i64)) == (0 as i64) {
                _yields.push(i);
            }
            i = i + (1 as i64);
        }
        __sifr_generator_iter = _yields.into_iter();
        __sifr_generator_initialized = true;
    }
    return __sifr_generator_iter.next();
}));
}

fn main() {
    let fibs: Vec<i64> = fibonacci(8 as i64).collect::<Vec<_>>();
    println!("{:?}", fibs);
    let even_nums: Vec<i64> = evens(10 as i64).collect::<Vec<_>>();
    println!("{:?}", even_nums);
    let nums: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64, 4 as i64, 5 as i64];
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
