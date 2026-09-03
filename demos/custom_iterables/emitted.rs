// src/main.rs
use ::sifr_runtime::SifrInt;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CountdownIter {
    current: SifrInt,
}
impl CountdownIter {
    fn new(start: SifrInt) -> Self {
        let sifr_generated_field_value_2a2e8a5afcc8d89a_63757272656e74: SifrInt = start.clone();
        Self {
            current: sifr_generated_field_value_2a2e8a5afcc8d89a_63757272656e74,
        }
    }
}
impl CountdownIter {
    fn sifr_generated_next__(&mut self) -> Option<SifrInt> {
        if &self.current.clone() <= &SifrInt::from_i64(0) {
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
        let sifr_generated_field_value_ee5d97ad45ad251f_7374617274: SifrInt = start.clone();
        Self {
            start: sifr_generated_field_value_ee5d97ad45ad251f_7374617274,
        }
    }
}
impl Countdown {
    fn sifr_generated_iter__(&self) -> Box<dyn Iterator<Item = SifrInt>> {
        let mut values: Vec<SifrInt> = Vec::new();
        let mut i: SifrInt = self.start.clone();
        while &i > &SifrInt::from_i64(0) {
            values.push(i.clone());
            i = &i - &SifrInt::from_i64(1);
        }
        Box::new(values.into_iter())
    }
}
impl Countdown {
    fn sifr_generated_reversed__(&self) -> Box<dyn Iterator<Item = SifrInt>> {
        let mut values: Vec<SifrInt> = Vec::new();
        let mut i: SifrInt = SifrInt::from_i64(1);
        while &i <= &self.start.clone() {
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
    println!(
        "{:?}",
        countdown
            .clone()
            .sifr_generated_iter__()
            .collect::<Vec<_>>()
    );
    println!(
        "{:?}",
        Box::new(countdown.clone().sifr_generated_reversed__()).collect::<Vec<_>>()
    );
    let mut running_total: SifrInt = SifrInt::from_i64(0);
    for value in Countdown::new(SifrInt::from_i64(4)).sifr_generated_iter__() {
        running_total = &running_total + &value;
    }
    println!("{running_total}");
    let mut it: CountdownIter = CountdownIter::new(SifrInt::from_i64(2));
    println!(
        "{}",
        it.sifr_generated_next__().map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
    println!(
        "{}",
        it.sifr_generated_next__().map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
    println!(
        "{}",
        it.sifr_generated_next__().map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
}
