fn drain(values: &mut Vec<i64>) -> i64 {
    let mut total: i64 = 0 as i64;
    while !values.is_empty() {
        let item: i64 = {
    let Some(__sifr_nonempty_pop_value) = values.pop() else {
        unreachable!("compiler-verified non-empty pop should return Some");
    };
    __sifr_nonempty_pop_value
};
        total = total + item;
    }
    return total;
}

fn drain_front(values: &mut Vec<i64>) -> i64 {
    let mut total: i64 = 0 as i64;
    while !values.is_empty() {
        let item: i64 = {
    let Some(__sifr_nonempty_pop_value) = ({
    let __len = values.len() as i64;
    let __index = {
    let __bound = 0 as i64;
    if __bound < 0 { (__len + __bound).max(0).min(__len) } else { __bound.min(__len) }
};
    if (__index < 0) || (__index >= __len) { None } else { Some(values.remove(__index as usize)) }
}) else {
        unreachable!("compiler-verified non-empty pop should return Some");
    };
    __sifr_nonempty_pop_value
};
        total = total + item;
    }
    return total;
}

fn main() {
    assert!(drain(&mut vec![1 as i64, 2 as i64, 3 as i64, 4 as i64]) == (10 as i64));
    assert!(drain(&mut vec![]) == (0 as i64));
    assert!(drain_front(&mut vec![1 as i64, 2 as i64, 3 as i64, 4 as i64]) == (10 as i64));
    assert!(drain_front(&mut vec![]) == (0 as i64));
}
