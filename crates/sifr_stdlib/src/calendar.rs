use sifr_runtime::interop::SifrIntBridge;

#[must_use]
pub fn calendar_isleap(year: SifrIntBridge) -> bool {
    let year = year.to_i64_saturating();
    is_leap_year_i64(year)
}

const fn is_leap_year_i64(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[must_use]
pub fn calendar_weekday(
    year: SifrIntBridge,
    month: SifrIntBridge,
    day: SifrIntBridge,
) -> SifrIntBridge {
    let year = year.to_i64_saturating();
    let month = month.to_i64_saturating();
    let day = day.to_i64_saturating();
    SifrIntBridge::from(calendar_weekday_i64(year, month, day))
}

#[must_use]
pub fn calendar_monthrange(year: SifrIntBridge, month: SifrIntBridge) -> Vec<SifrIntBridge> {
    let year = year.to_i64_saturating();
    let month = month.to_i64_saturating();
    vec![
        SifrIntBridge::from(calendar_weekday_i64(year, month, 1)),
        SifrIntBridge::from(days_in_month(year, month)),
    ]
}

fn calendar_weekday_i64(year: i64, month: i64, day: i64) -> i64 {
    let year = i128::from(year);
    let month = i128::from(month);
    let day = i128::from(day);
    let y = if month < 3 { year - 1 } else { year };
    let weekday = y + y / 4 - y / 100 + y / 400 + month_offset(month) + day + 6;
    i64::try_from(weekday.rem_euclid(7)).unwrap_or(0)
}

const fn month_offset(month: i128) -> i128 {
    match month {
        1 => 0,
        2 => 3,
        3 => 2,
        4 => 5,
        5 => 0,
        6 => 3,
        7 => 5,
        8 => 1,
        9 => 4,
        10 => 6,
        11 => 2,
        12 => 4,
        _ => 0,
    }
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year_i64(year) => 29,
        2 => 28,
        _ => 30,
    }
}
