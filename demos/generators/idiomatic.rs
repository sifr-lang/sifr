struct Timer {
    label: String,
}

impl Timer {
    fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
        }
    }

    fn enter(&self) -> TimerGuard<'_> {
        let _ = self.label.as_str();
        TimerGuard { timer: self }
    }

    fn exit(&self) {
        let _ = self.label.as_str();
    }
}

struct TimerGuard<'a> {
    timer: &'a Timer,
}

impl Drop for TimerGuard<'_> {
    fn drop(&mut self) {
        self.timer.exit();
    }
}

fn fibonacci(n: usize) -> impl Iterator<Item = i64> {
    std::iter::successors(Some((0_i64, 1_i64)), |(a, b)| Some((*b, *a + *b)))
        .map(|(value, _)| value)
        .take(n)
}

fn evens(limit: i64) -> impl Iterator<Item = i64> {
    (0..limit).filter(|value| value % 2 == 0)
}

fn main() {
    let fibs: Vec<i64> = fibonacci(8).collect();
    println!("{fibs:?}");

    let even_nums: Vec<i64> = evens(10).collect();
    println!("{even_nums:?}");

    for x in [1_i64, 2, 3, 4, 5] {
        println!("{}", x * x);
    }

    let timer = Timer::new("work");
    {
        let _t = timer.enter();
        println!("doing work");
    }
    println!("done");
}
