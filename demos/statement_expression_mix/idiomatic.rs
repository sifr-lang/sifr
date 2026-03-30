fn main() {
    let mut acc = 0;

    for n in [1_i64, 2, 3] {
        acc += n;
    }
    acc += 1;

    let mut i = 0;
    while i < 3 {
        acc += i;
        i += 1;
    }
    acc += 2;

    let ready = true;
    if ready {
        acc += 10;
    } else {
        acc += 100;
    }

    assert!(acc > 0);
    let line = format!("acc = {acc}");
    println!("{line}");
    assert_eq!(line, "acc = 22");
}
