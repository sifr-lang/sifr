#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Counter {
    count: i64,
}

impl Counter {
    fn new(count: i64) -> Self {
        return Self { count: count };
    }
    fn increment(&mut self) {
        self.count += 1 as i64;
    }
}

impl std::fmt::Display for Counter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Counter(count={})", self.count);
    }
}

fn main() {
    let mut matrix = vec![vec![0, 0, 0], vec![0, 0, 0], vec![0, 0, 0]];
    matrix[0][0] = 1;
    matrix[1][1] = 1;
    matrix[2][2] = 1;
    println!("{}", matrix[0][0]);
    println!("{}", matrix[1][1]);
    println!("{}", matrix[2][2]);

    let mut scores = vec![10, 20, 30];
    scores[0] += 5;
    scores[1] -= 3;
    scores[2] *= 2;

    let s0 = scores.first().copied();
    let s1 = scores.get(1).copied();
    let s2 = scores.get(2).copied();

    if let Some(s0) = s0 {
        println!("{}", s0);
    }
    if let Some(s1) = s1 {
        println!("{}", s1);
    }
    if let Some(s2) = s2 {
        println!("{}", s2);
    }
    let mut c = Counter::new(0 as i64);
    c.increment();
    c.increment();
    c.increment();
    println!("{}", c.count);
}
