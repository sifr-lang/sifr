use std::collections::BTreeMap;

fn assert_ok<T, E>(value: Result<T, E>) {
    assert!(value.is_ok());
}

fn assert_err<T, E>(value: Result<T, E>) {
    assert!(value.is_err());
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

fn negate(x: i64) -> i64 {
    -x
}

fn add(x: i64, y: i64) -> i64 {
    x + y
}

fn ord_text(text: &str) -> Result<i64, ValueError> {
    let chars = text.chars().collect::<Vec<_>>();
    if chars.len() == 1 {
        Ok(i64::from(chars[0] as u32))
    } else {
        Err(ValueError::new("ord() expected a string of length 1"))
    }
}

fn chr_codepoint(codepoint: i64) -> Result<String, ValueError> {
    if !(0..=0x10_FFFF).contains(&codepoint) {
        return Err(ValueError::new("chr() arg not in range(0x110000)"));
    }

    char::from_u32(codepoint as u32)
        .map(|value| value.to_string())
        .ok_or_else(|| ValueError::new("chr() arg not in range(0x110000)"))
}

fn main() {
    println!("=== constructors ===");
    println!(
        "{:?}",
        "sifr".chars().map(|ch| ch.to_string()).collect::<Vec<_>>()
    );
    println!("{:?}", (1_i64, 2_i64, 3_i64));

    let mut demo_dict = BTreeMap::from([("compiler".to_string(), 1_i64)]);
    demo_dict.insert("demo".to_string(), 2_i64);
    println!("{demo_dict:?}");

    println!("=== helpers ===");

    let mut plain_sorted = vec![3_i64, 1_i64, 2_i64];
    plain_sorted.sort_unstable();
    println!("{plain_sorted:?}");

    let mut key_sorted = vec![3_i64, 1_i64, 2_i64];
    key_sorted.sort_by_key(|value| negate(*value));
    println!("{key_sorted:?}");

    let mut reverse_sorted = vec![3_i64, 1_i64, 2_i64];
    reverse_sorted.sort_unstable_by(|left, right| right.cmp(left));
    println!("{reverse_sorted:?}");

    println!(
        "{:?}",
        "sifr"
            .chars()
            .rev()
            .map(|ch| ch.to_string())
            .collect::<Vec<_>>()
    );
    println!(
        "{:?}",
        vec!["a".to_string(), "b".to_string()]
            .into_iter()
            .enumerate()
            .map(|(index, value)| (index as i64 + 10, value))
            .collect::<Vec<_>>()
    );
    println!(
        "{:?}",
        vec![1_i64, 2]
            .into_iter()
            .zip(vec!["a".to_string(), "b".to_string()])
            .zip(vec![true, false])
            .map(|((number, text), flag)| (number, text, flag))
            .collect::<Vec<_>>()
    );
    println!(
        "{:?}",
        vec![1_i64, 2, 3]
            .into_iter()
            .zip(vec![4_i64, 5, 6])
            .map(|(left, right)| add(left, right))
            .collect::<Vec<_>>()
    );
    println!("{:?}", (2_i64..9).step_by(3).collect::<Vec<_>>());

    println!("=== ord/chr ===");
    println!("{}", ord_text("A").unwrap_or_default());
    println!("{}", chr_codepoint(66).unwrap_or_default());

    let ok_text = "Z";
    let bad_text = "ZZ";
    let ok_codepoint = 67_i64;
    let huge = 1_114_112_i64;

    assert_ok(ord_text(ok_text));
    assert_err(ord_text(bad_text));
    assert_ok(chr_codepoint(ok_codepoint));
    assert_err(chr_codepoint(huge));
}
