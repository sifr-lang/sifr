fn evaluate(seed: i64) -> i64 {
    let mut total = 0;

    for n in 0..seed {
        if n == 1 {
            continue;
        }
        if n == 6 {
            break;
        }
        if n % 2 == 0 {
            total += n;
        } else {
            total += 1;
        }
    }

    total
}

fn safe(seed: i64) -> i64 {
    let value = evaluate(seed);
    if value > 3 {
        value
    } else {
        42
    }
}

#[allow(unreachable_code)]
fn unreachable_tail() -> i64 {
    return 9;
    10
}

fn test_cfg_flow_matrix() {
    assert_eq!(safe(8), 8);
    assert_eq!(safe(3), 42);
    assert_eq!(unreachable_tail(), 9);
}

fn main() {
    test_cfg_flow_matrix();
    println!("cfg flow activation regression matrix demo:");
    println!("{}", safe(8));
    println!("{}", safe(3));
    println!("{}", unreachable_tail());
}
