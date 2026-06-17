// Reference: generators
// Reference: generators
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.12s
     Running `target/debug/sifr emit demos/generators_demo.sifr`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Timer {
    label: String,
}

impl Timer {
    fn new(label: String) -> Self {
        Self {
            label: label,
        }
    }

}

fn fibonacci(n: i64) -> Vec<i64> {
    let mut _yields: Vec<i64> = Vec::new();
    let mut a: i64 = 0_i64;
    let mut b: i64 = 1_i64;
    let mut i: i64 = 0_i64;
    while i < n {
        _yields.push(a);
        let temp: i64 = a + b;
        a = b;
        b = temp;
        i = i + 1_i64;
    }
    _yields
}

fn evens(limit: i64) -> Vec<i64> {
    let mut _yields: Vec<i64> = Vec::new();
    let mut i: i64 = 0_i64;
    while i < limit {
        if i % 2_i64 == 0_i64 {
            _yields.push(i);
        }
        i = i + 1_i64;
    }
    _yields
}

fn main() {
    let fibs: Vec<i64> = fibonacci(8_i64);
    println!("{:?}", fibs);
    println!("{:?}", evens(10_i64));
    let nums: Vec<i64> = vec![1_i64, 2_i64, 3_i64, 4_i64, 5_i64];
    for sq in nums.clone().into_iter().map(|x| x * x) {
        println!("{}", sq);
    }
    {
        let t = Timer::new("work".to_string());
        println!("{}", "doing work");
    }
    println!("{}", "done");
}
