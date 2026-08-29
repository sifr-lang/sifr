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

fn fibonacci(n: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<SifrInt> = Vec::new().into_iter();
    Box::new(::std::iter::from_fn(move || {
    if !__sifr_generator_initialized {
        let mut _yields: Vec<SifrInt> = Vec::new();
        let mut a: SifrInt = SifrInt::from_i64(0);
        let mut b: SifrInt = SifrInt::from_i64(1);
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &n {
            _yields.push(a.clone());
            let temp: SifrInt = &a + &b;
            a = b.clone();
            b = temp.clone();
            i = &i + &SifrInt::from_i64(1);
        }
        __sifr_generator_iter = _yields.into_iter();
        __sifr_generator_initialized = true;
    }
    __sifr_generator_iter.next()
}))
}

fn evens(limit: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<SifrInt> = Vec::new().into_iter();
    Box::new(::std::iter::from_fn(move || {
    if !__sifr_generator_initialized {
        let mut _yields: Vec<SifrInt> = Vec::new();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &limit {
            if (&i.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0)) {
                _yields.push(i.clone());
            }
            i = &i + &SifrInt::from_i64(1);
        }
        __sifr_generator_iter = _yields.into_iter();
        __sifr_generator_initialized = true;
    }
    __sifr_generator_iter.next()
}))
}

fn main() {
    let fibs: Vec<SifrInt> = fibonacci(SifrInt::from_i64(8)).collect::<Vec<_>>();
    println!("{:?}", fibs);
    let even_nums: Vec<SifrInt> = evens(SifrInt::from_i64(10)).collect::<Vec<_>>();
    println!("{:?}", even_nums);
    let nums: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4), SifrInt::from_i64(5)];
    for x in nums.iter().cloned() {
        println!("{}", &x * &x);
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
