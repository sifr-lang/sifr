struct CountdownIter {
    current: i64,
}

impl CountdownIter {
    fn new(start: i64) -> Self {
        Self { current: start }
    }
}

impl Iterator for CountdownIter {
    type Item = i64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current <= 0 {
            return None;
        }

        let value = self.current;
        self.current -= 1;
        Some(value)
    }
}

struct Countdown {
    start: i64,
}

impl Countdown {
    fn new(start: i64) -> Self {
        Self { start }
    }

    fn reversed(&self) -> impl Iterator<Item = i64> {
        1..=self.start
    }
}

impl IntoIterator for Countdown {
    type Item = i64;
    type IntoIter = std::iter::Rev<std::ops::RangeInclusive<i64>>;

    fn into_iter(self) -> Self::IntoIter {
        (1..=self.start).rev()
    }
}

fn format_optional(value: Option<i64>) -> String {
    value.map_or_else(|| "None".to_string(), |item| item.to_string())
}

fn main() {
    let countdown = Countdown::new(4);
    println!("{:?}", countdown.into_iter().collect::<Vec<_>>());

    let countdown = Countdown::new(4);
    println!("{:?}", countdown.reversed().collect::<Vec<_>>());

    let running_total: i64 = Countdown::new(4).into_iter().sum();
    println!("{running_total}");

    let mut iter = CountdownIter::new(2);
    println!("{}", format_optional(iter.next()));
    println!("{}", format_optional(iter.next()));
    println!("{}", format_optional(iter.next()));
}
