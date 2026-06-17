fn gen(flag: bool) -> impl Iterator<Item = i64> {
    let mut i = 0_i64;
    let mut emitted_else = false;

    std::iter::from_fn(move || {
        while i < 2 {
            if flag && i == 0 {
                i = 2;
                break;
            }

            let value = i;
            i += 1;
            return Some(value);
        }

        if !flag && i == 2 && !emitted_else {
            emitted_else = true;
            return Some(99);
        }

        None
    })
}

fn main() {
    println!("generator_break_else yield/loop-path coverage demo:");
    for value in gen(false) {
        println!("{value}");
    }
    for value in gen(true) {
        println!("{value}");
    }
}
