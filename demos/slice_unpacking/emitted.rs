// src/main.rs
use ::std::collections::HashMap;

use ::sifr_runtime::SifrInt;

static __SIFR_HOISTED_DICT_0: ::std::sync::LazyLock<HashMap<String, SifrInt>> = ::std::sync::LazyLock::new(|| HashMap::from([("x".to_string(), SifrInt::from_i64(11)), ("y".to_string(), SifrInt::from_i64(22))]));

fn main() {
    let nums: Vec<SifrInt> = vec![SifrInt::from_i64(3), SifrInt::from_i64(6), SifrInt::from_i64(9), SifrInt::from_i64(12)];
    println!("{}", ({
    let __sifr_index_list = &nums;
    let __sifr_index_i = SifrInt::from_i64(0);
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list.len());
    __sifr_index_list.get(__sifr_index_norm).cloned()
}).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", ({
    let __sifr_index_list = &nums;
    let __sifr_index_i = SifrInt::from_i64(99);
    let __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list.len());
    __sifr_index_list.get(__sifr_index_norm).cloned()
}).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    let scores = &*__SIFR_HOISTED_DICT_0;
    println!("{}", (scores.get("x").cloned()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", (scores.get("z").cloned()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    let _star_tmp = &nums;
    let a = _star_tmp[0].clone();
    let mid = _star_tmp[1.._star_tmp.len() - 1].to_vec();
    let b = _star_tmp[_star_tmp.len() - 1].clone();
    println!("{}", a);
    println!("{:?}", mid);
    println!("{}", b);
    println!("{:?}", {
    let _v = &(nums);
    let _len = _v.len();
    ::sifr_runtime::SifrSliceIndices::new_known_nonzero(_len, None, None, &SifrInt::from_i64(2)).map(|_i| _v[_i].clone()).collect::<Vec<_>>()
});
    println!("{}", SifrInt::from(nums.len()));
    println!("clone_slice_unpacking_slice_unpack_demo: pass");
}
