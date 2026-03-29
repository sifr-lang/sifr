use chrono::{Datelike, Local, NaiveDateTime};

#[derive(Clone, Copy)]
struct Timezone {
    offset_seconds: i32,
}

impl Timezone {
    fn new(offset_seconds: i32) -> Self {
        Self { offset_seconds }
    }
}

impl std::fmt::Display for Timezone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.offset_seconds == 0 {
            return write!(f, "UTC");
        }

        let sign = if self.offset_seconds >= 0 { '+' } else { '-' };
        let absolute = self.offset_seconds.abs();
        let hours = absolute / 3_600;
        let minutes = (absolute % 3_600) / 60;
        write!(f, "UTC{sign}{hours:02}:{minutes:02}")
    }
}

struct Datetime {
    naive: NaiveDateTime,
}

impl Datetime {
    fn isoformat(&self) -> String {
        self.naive.format("%Y-%m-%dT%H:%M:%S").to_string()
    }

    fn year(&self) -> i32 {
        self.naive.date().year()
    }

    fn month(&self) -> u32 {
        self.naive.date().month()
    }

    fn day(&self) -> u32 {
        self.naive.date().day()
    }
}

fn timezone(offset_seconds: i32) -> Timezone {
    Timezone::new(offset_seconds)
}

fn now() -> Datetime {
    Datetime {
        naive: Local::now().naive_local(),
    }
}

fn main() {
    let zero = timezone(0);
    assert_eq!(zero.to_string(), "UTC");

    let plus_two_thirty = timezone(9_000);
    let minus_five = timezone(-18_000);
    assert_eq!(plus_two_thirty.to_string(), "UTC+02:30");
    assert_eq!(minus_five.to_string(), "UTC-05:00");

    let current = now();
    assert!(current.year() >= 1970);
    assert!((1..=12).contains(&current.month()));
    assert!((1..=31).contains(&current.day()));
    assert_eq!(current.isoformat().len(), 19);
}
