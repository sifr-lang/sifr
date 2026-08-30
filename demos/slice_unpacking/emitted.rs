// src/main.rs
mod __sifr_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ValueError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ValueError {}
}
pub use __sifr_project_nominals::ValueError;
use ::std::collections::HashMap;
use ::sifr_runtime::SifrInt;
static __SIFR_HOISTED_DICT_0: ::std::sync::LazyLock<HashMap<String, SifrInt>> = ::std::sync::LazyLock::new(||
HashMap::from([
    ("x".to_string(), SifrInt::from_i64(11)),
    ("y".to_string(), SifrInt::from_i64(22)),
]));
fn main() {
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(3), SifrInt::from_i64(6), SifrInt::from_i64(9),
        SifrInt::from_i64(12)
    ];
    println!(
        "{}", ({ let __sifr_index_list = & nums; let __sifr_index_i =
        SifrInt::from_i64(0); let __sifr_index_norm = __sifr_index_i
        .normalize_index_or_len(__sifr_index_list.len()); __sifr_index_list
        .get(__sifr_index_norm).cloned() }).map_or("None".to_string().to_string(), | __v
        | format!("{}", __v))
    );
    println!(
        "{}", ({ let __sifr_index_list = & nums; let __sifr_index_i =
        SifrInt::from_i64(99); let __sifr_index_norm = __sifr_index_i
        .normalize_index_or_len(__sifr_index_list.len()); __sifr_index_list
        .get(__sifr_index_norm).cloned() }).map_or("None".to_string().to_string(), | __v
        | format!("{}", __v))
    );
    let scores = &*__SIFR_HOISTED_DICT_0;
    println!(
        "{}", (scores.get("x").cloned()).map_or("None".to_string().to_string(), | __v |
        format!("{}", __v))
    );
    println!(
        "{}", (scores.get("z").cloned()).map_or("None".to_string().to_string(), | __v |
        format!("{}", __v))
    );
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let __sifr_unpack_source = &nums;
        let [__sifr_before_0, __sifr_star @ .., __sifr_after_0] = __sifr_unpack_source
            .as_slice() else {
            return Err(ValueError::new("not enough values to unpack".to_string()));
        };
        let a = __sifr_before_0.clone();
        let mid = __sifr_star.to_vec();
        let b = __sifr_after_0.clone();
        println!("{}", a);
        println!("{:?}", mid);
        println!("{}", b);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let error = __sifr_try_err.clone();
        println!("{}", error.message.clone());
    }
    println!(
        "{:?}", { let _v = & (nums); let _len = _v.len();
        ::sifr_runtime::SifrSliceIndices::new_known_nonzero(_len, None, None, &
        SifrInt::from_i64(2)).filter_map(| _i | _v.get(_i).cloned()).collect::< Vec < _
        >> () }
    );
    println!("{}", SifrInt::from(nums.len()));
    println!("clone_slice_unpacking_slice_unpack_demo: pass");
}
