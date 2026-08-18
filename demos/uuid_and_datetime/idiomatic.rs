use chrono::{DateTime, FixedOffset, Utc};
use uuid::Uuid;

struct SimpleUuid(Uuid);

impl SimpleUuid {
    fn version(&self) -> usize {
        self.0.get_version_num() as usize
    }
}

fn namespace_dns() -> SimpleUuid {
    SimpleUuid(Uuid::NAMESPACE_DNS)
}

fn uuid3(namespace: &SimpleUuid, name: &str) -> SimpleUuid {
    SimpleUuid(Uuid::new_v3(&namespace.0, name.as_bytes()))
}

fn uuid5(namespace: &SimpleUuid, name: &str) -> SimpleUuid {
    SimpleUuid(Uuid::new_v5(&namespace.0, name.as_bytes()))
}

#[derive(Clone, Copy)]
struct Timezone {
    offset_seconds: i32,
}

impl Timezone {
    fn new(offset_seconds: i32) -> Self {
        Self { offset_seconds }
    }

    fn fixed_offset(self) -> FixedOffset {
        FixedOffset::east_opt(self.offset_seconds).expect("valid fixed offset")
    }
}

struct Datetime {
    inner: DateTime<FixedOffset>,
}

impl Datetime {
    fn isoformat(&self) -> String {
        self.inner.format("%Y-%m-%dT%H:%M:%S%:z").to_string()
    }
}

fn utc_timezone() -> Timezone {
    Timezone::new(0)
}

fn timezone(offset_seconds: i32) -> Timezone {
    Timezone::new(offset_seconds)
}

fn now(tz: Timezone) -> Datetime {
    Datetime {
        inner: Utc::now().with_timezone(&tz.fixed_offset()),
    }
}

fn from_timestamp(seconds: f64, tz: Timezone) -> Result<Datetime, String> {
    let secs = seconds.trunc() as i64;
    let nanos = ((seconds.fract() * 1_000_000_000.0).round()) as u32;
    let utc = DateTime::<Utc>::from_timestamp(secs, nanos)
        .ok_or_else(|| "timestamp out of range".to_string())?;
    Ok(Datetime {
        inner: utc.with_timezone(&tz.fixed_offset()),
    })
}

fn main() {
    let name_v3 = uuid3(&namespace_dns(), "sifr.sh");
    let name_v5 = uuid5(&namespace_dns(), "sifr.sh");
    assert_eq!(name_v3.version(), 3);
    assert_eq!(name_v5.version(), 5);

    let utc_now = now(utc_timezone());
    assert!(utc_now.isoformat().ends_with("+00:00"));

    let plus_two = timezone(7_200);
    let epoch_shifted = from_timestamp(0.0, plus_two).expect("valid timestamp");
    assert_eq!(epoch_shifted.isoformat(), "1970-01-01T02:00:00+02:00");
}
