// src/main.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CountdownIter {
    current: i64,
}

impl CountdownIter {
    fn new(start: i64) -> Self {
        let __sifr_field_init_0: i64 = start;
        Self { current: __sifr_field_init_0 }
    }
}

impl CountdownIter {
    fn __iter__(&self) -> Box<dyn Iterator<Item = i64>> {
        Box::new(vec![self.current].into_iter())
    }
}

impl CountdownIter {
    fn __next__(&mut self) -> Option<i64> {
        if (self.current <= (0_i64)) {
            return None;
        }
        let value: i64 = self.current;
        self.current -= 1_i64;
        Some(value)
    }
}

impl ::std::fmt::Display for CountdownIter {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "CountdownIter(current={})", self.current)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Countdown {
    start: i64,
}

impl Countdown {
    fn new(start: i64) -> Self {
        let __sifr_field_init_0: i64 = start;
        Self { start: __sifr_field_init_0 }
    }
}

impl Countdown {
    fn __iter__(&self) -> Box<dyn Iterator<Item = i64>> {
        let mut values: Vec<i64> = vec![];
        let mut i: i64 = self.start;
        while i > (0_i64) {
            values.push(i);
            i -= 1_i64;
        }
        Box::new(values.into_iter())
    }
}

impl Countdown {
    fn __reversed__(&self) -> Box<dyn Iterator<Item = i64>> {
        let mut values: Vec<i64> = vec![];
        let mut i: i64 = 1_i64;
        while (i <= self.start) {
            values.push(i);
            i += 1_i64;
        }
        Box::new(values.into_iter())
    }
}

impl ::std::fmt::Display for Countdown {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Countdown(start={})", self.start)
    }
}

fn main() {
    let countdown: Countdown = Countdown::new(4_i64);
    println!("{:?}", (countdown).clone().__iter__().collect::<Vec<_>>());
    println!("{:?}", Box::new((countdown).clone().__reversed__().into_iter()).collect::<Vec<_>>());
    let mut running_total: i64 = 0_i64;
    for value in Countdown::new(4_i64).__iter__() {
        running_total += value;
    }
    println!("{}", running_total);
    let mut it: CountdownIter = CountdownIter::new(2_i64);
    println!("{}", (it.__next__()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", (it.__next__()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", (it.__next__()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
}
