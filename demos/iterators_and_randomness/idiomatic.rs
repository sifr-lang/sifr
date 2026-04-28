use rand::prelude::{IndexedRandom, SliceRandom};
use std::fmt::{self, Display};

#[derive(Debug, Clone)]
struct ValueError {
    message: String,
}

impl Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for ValueError {}

#[derive(Debug, Clone, Copy, Default)]
struct Doubler;

impl Doubler {
    fn apply(self, x: i64) -> i64 {
        x * 2
    }
}

fn add(a: i64, b: i64) -> i64 {
    a + b
}

fn product_repeat(values: &[i64], repeat: usize) -> Vec<Vec<i64>> {
    let mut result = vec![Vec::new()];
    for _ in 0..repeat {
        let mut next = Vec::new();
        for prefix in &result {
            for &value in values {
                let mut item = prefix.clone();
                item.push(value);
                next.push(item);
            }
        }
        result = next;
    }
    result
}

fn permutations_of_two(values: &[i64]) -> Vec<Vec<i64>> {
    let mut result = Vec::new();
    for (i, &left) in values.iter().enumerate() {
        for (j, &right) in values.iter().enumerate() {
            if i != j {
                result.push(vec![left, right]);
            }
        }
    }
    result
}

fn combinations_of_two(values: &[i64]) -> Vec<Vec<i64>> {
    let mut result = Vec::new();
    for i in 0..values.len() {
        for j in i + 1..values.len() {
            result.push(vec![values[i], values[j]]);
        }
    }
    result
}

fn choice(values: &[i64]) -> Result<i64, ValueError> {
    values
        .choose(&mut rand::rng())
        .copied()
        .ok_or_else(|| ValueError {
            message: "choice from empty sequence".to_string(),
        })
}

fn choices(values: &[i64], k: usize) -> Result<Vec<i64>, ValueError> {
    if values.is_empty() {
        return Err(ValueError {
            message: "choices from empty sequence".to_string(),
        });
    }
    let mut rng = rand::rng();
    Ok((0..k)
        .map(|_| {
            let index = rand::RngExt::random_range(&mut rng, 0..values.len());
            values[index]
        })
        .collect())
}

fn randrange(stop: i64) -> Result<i64, ValueError> {
    if stop <= 0 {
        return Err(ValueError {
            message: "empty randrange".to_string(),
        });
    }
    Ok(rand::RngExt::random_range(&mut rand::rng(), 0..stop))
}

fn compare_digest(left: &str, right: &str) -> bool {
    let left = left.as_bytes();
    let right = right.as_bytes();
    let mut diff = left.len() ^ right.len();
    for i in 0..left.len().max(right.len()) {
        let a = left.get(i).copied().unwrap_or(0);
        let b = right.get(i).copied().unwrap_or(0);
        diff |= usize::from(a ^ b);
    }
    diff == 0
}

fn token_hex(nbytes: usize) -> String {
    let mut rng = rand::rng();
    (0..nbytes)
        .map(|_| format!("{:02x}", rand::RngExt::random::<u8>(&mut rng)))
        .collect()
}

fn randbits(bits: u32) -> Result<i64, ValueError> {
    if bits > 62 {
        return Err(ValueError {
            message: "randbits: number of bits must be <= 62".to_string(),
        });
    }
    let mut rng = rand::rng();
    let mut value = 0_i64;
    for _ in 0..bits {
        value = (value << 1) | i64::from(rand::RngExt::random_range(&mut rng, 0..=1_u8));
    }
    Ok(value)
}

fn main() {
    let chain_values = [vec![1], vec![2], vec![3], vec![4]]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    println!("chain(*iterables) = {:?}", chain_values);

    let sliced = [10, 20, 30, 40, 50]
        .iter()
        .skip(1)
        .take(4)
        .step_by(2)
        .copied()
        .collect::<Vec<_>>();
    println!("islice(start, stop, step) = {:?}", sliced);

    println!("product(repeat=2) = {:?}", product_repeat(&[1, 2], 2));
    println!("permutations(r=2) = {:?}", permutations_of_two(&[1, 2, 3]));
    println!("combinations(r=2) = {:?}", combinations_of_two(&[1, 2, 3]));

    let starmapped = [(2, 3), (4, 5)]
        .into_iter()
        .map(|(a, b)| add(a, b))
        .collect::<Vec<_>>();
    println!("starmap(add, pairs) = {:?}", starmapped);

    let doubler = Doubler;
    println!("callable object direct = {}", doubler.apply(4));

    let mut items = vec![1, 2, 3, 4, 5];
    items.shuffle(&mut rand::rng());
    println!("shuffle(mut items) len = {}", items.len());

    match (choice(&items), choices(&items, 3), randrange(10)) {
        (Ok(picked), Ok(many), Ok(rr)) => {
            println!("choice(items) ok = {}", (1..=5).contains(&picked));
            println!("choices(items, k=3) len = {}", many.len());
            println!("randrange(10) ok = {}", (0..10).contains(&rr));
        }
        (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => {
            println!("random error: {}", error.message);
        }
    }

    println!("secrets.compare_digest = {}", compare_digest("abc", "abc"));
    println!("secrets.token_hex(4) len = {}", token_hex(4).len());
    match randbits(16) {
        Ok(bits) => println!("secrets.randbits(16) ok = {}", bits >= 0),
        Err(error) => println!("secrets error: {}", error.message),
    }
}
