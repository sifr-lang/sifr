// src/main.rs
use ::sifr_runtime::SifrInt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CountdownIter {
    current: SifrInt,
}

impl CountdownIter {
    fn new(start: SifrInt) -> Self {
        let __sifr_field_init_0: SifrInt = start.clone();
        Self { current: __sifr_field_init_0 }
    }
}

impl CountdownIter {
    fn __iter__(&self) -> Box<dyn Iterator<Item = SifrInt>> {
        Box::new(vec![self.current.clone()].into_iter())
    }
}

impl CountdownIter {
    fn __next__(&mut self) -> Option<SifrInt> {
        if (&self.current.clone() <= &SifrInt::from_i64(0)) {
            return None;
        }
        let value: SifrInt = self.current.clone();
        self.current = &self.current.clone() - &SifrInt::from_i64(1);
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
    start: SifrInt,
}

impl Countdown {
    fn new(start: SifrInt) -> Self {
        let __sifr_field_init_0: SifrInt = start.clone();
        Self { start: __sifr_field_init_0 }
    }
}

impl Countdown {
    fn __iter__(&self) -> Box<dyn Iterator<Item = SifrInt>> {
        let mut values: Vec<SifrInt> = vec![];
        let mut i: SifrInt = self.start.clone();
        while (&i > &SifrInt::from_i64(0)) {
            values.push(i.clone());
            i = &i - &SifrInt::from_i64(1);
        }
        Box::new(values.into_iter())
    }
}

impl Countdown {
    fn __reversed__(&self) -> Box<dyn Iterator<Item = SifrInt>> {
        let mut values: Vec<SifrInt> = vec![];
        let mut i: SifrInt = SifrInt::from_i64(1);
        while (&i <= &self.start.clone()) {
            values.push(i.clone());
            i = &i + &SifrInt::from_i64(1);
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
    let countdown: Countdown = Countdown::new(SifrInt::from_i64(4));
    println!("{:?}", (countdown).clone().__iter__().collect::<Vec<_>>());
    println!("{:?}", Box::new((countdown).clone().__reversed__().into_iter()).collect::<Vec<_>>());
    let mut running_total: SifrInt = SifrInt::from_i64(0);
    for value in Countdown::new(SifrInt::from_i64(4)).__iter__() {
        running_total = &running_total + &value;
    }
    println!("{}", running_total);
    let mut it: CountdownIter = CountdownIter::new(SifrInt::from_i64(2));
    println!("{}", (it.__next__()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", (it.__next__()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", (it.__next__()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
}
