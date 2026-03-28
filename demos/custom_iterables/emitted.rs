#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CountdownIter {
    current: i64,
}

impl CountdownIter {
    fn new(start: i64) -> Self {
        return Self { current: start };
    }
    fn __iter__(&self) -> Box<dyn Iterator<Item = i64>> {
        return Box::new((vec![self.current]).into_iter());
    }
    fn __next__(&mut self) -> Option<i64> {
        if self.current <= (0 as i64) {
            return None;
        }
        let value: i64 = self.current;
        self.current = self.current - (1 as i64);
        return Some(value);
    }
}

impl std::fmt::Display for CountdownIter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "CountdownIter(current={})", self.current);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Countdown {
    start: i64,
}

impl Countdown {
    fn new(start: i64) -> Self {
        return Self { start: start };
    }
    fn __iter__(&self) -> Box<dyn Iterator<Item = i64>> {
        let mut values: Vec<i64> = vec![];
        let mut i: i64 = self.start;
        while i > (0 as i64) {
            values.push(i);
            i = i - (1 as i64);
        }
        return Box::new((values).iter().copied());
    }
    fn __reversed__(&self) -> Box<dyn Iterator<Item = i64>> {
        let mut values: Vec<i64> = vec![];
        let mut i: i64 = 1 as i64;
        while i <= self.start {
            values.push(i);
            i = i + (1 as i64);
        }
        return Box::new((values).iter().copied());
    }
}

impl std::fmt::Display for Countdown {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Countdown(start={})", self.start);
    }
}

fn main() {
    let countdown: Countdown = Countdown::new(4 as i64);
    println!("{:?}", (countdown).clone().__iter__().collect::<Vec<_>>());
    println!("{:?}", Box::new((countdown).clone().__reversed__().into_iter()).collect::<Vec<_>>());
    let mut running_total: i64 = 0 as i64;
    for value in Countdown::new(4 as i64).__iter__() {
        running_total = running_total + value;
    }
    println!("{}", running_total);
    let mut it: CountdownIter = CountdownIter::new(2 as i64);
    println!("{}", (it.__next__()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", (it.__next__()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", (it.__next__()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
}
