use std::collections::HashMap;

fn main() {
    let nums: Vec<i64> = vec![3 as i64, 6 as i64, 9 as i64, 12 as i64];
    println!("{}", ({
    let __sifr_index_list = &nums;
    let __sifr_index_i = 0 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
}).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", ({
    let __sifr_index_list = &nums;
    let __sifr_index_i = 99 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
}).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    let scores: HashMap<String, i64> = HashMap::from([("x".to_string(), 11 as i64), ("y".to_string(), 22 as i64)]);
    println!("{}", (scores.get("x").copied()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", (scores.get("z").copied()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    let _star_tmp = &nums;
    let a = _star_tmp[0];
    let mid = _star_tmp[1.._star_tmp.len() - 1].to_vec();
    let b = _star_tmp[_star_tmp.len() - 1];
    println!("{}", a);
    println!("{:?}", mid);
    println!("{}", b);
    println!("{:?}", {
    let _v = &(nums);
    let _len = _v.len() as i64;
    let _step = 2 as i64;
    let _start = if _step > 0 { 0 as usize } else { (_len - 1) as usize };
    let _stop = if _step > 0 { _len as usize } else { usize::MAX };
    let mut _result = Vec::new();
    if _step > 0 {
        let mut _i = _start;
        while _i < _stop {
            if let Some(_el) = _v.get(_i) {
                _result.push(*_el);
            }
            _i += _step as usize;
        }
    } else {
        let mut _i = _start as i64;
        let _stop_i = _stop as i64;
        while _i > _stop_i {
            if _i >= 0 {
                if let Some(_el) = _v.get(_i as usize) {
                    _result.push(*_el);
                }
            }
            _i += _step;
        }
    }
    _result
});
    println!("{}", nums.len() as i64);
    println!("clone_slice_unpacking_slice_unpack_demo: pass");
}
