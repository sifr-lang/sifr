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
use ::sifr_runtime::SifrInt;
fn double(n: SifrInt) -> SifrInt {
    &n * &SifrInt::from_i64(2)
}
fn is_even(n: SifrInt) -> bool {
    (&n.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0))
}
fn main() {
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3),
        SifrInt::from_i64(4)
    ];
    let mapped: Vec<SifrInt> = Box::new(
            nums.iter().cloned().map(|__sifr_map_item| double(__sifr_map_item)),
        )
        .collect::<Vec<_>>();
    let filtered: Vec<SifrInt> = Box::new(
            nums
                .iter()
                .cloned()
                .filter(|__filter_item| {
                    let __filter_value = __filter_item.clone();
                    is_even(__filter_value)
                }),
        )
        .collect::<Vec<_>>();
    let mut first: SifrInt = SifrInt::from_i64(0);
    let mut rest: Vec<SifrInt> = vec![];
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let __sifr_unpack_source = &nums;
        let [__sifr_before_0, __sifr_star @ ..] = __sifr_unpack_source.as_slice() else {
            return Err(ValueError::new("not enough values to unpack".to_string()));
        };
        first = __sifr_before_0.clone();
        rest = __sifr_star.to_vec();
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let _e = __sifr_try_err.clone();
        assert!(false);
    }
    println!("{}", format!("{:?}", mapped));
    println!("{}", format!("{:?}", filtered));
    println!("{}", format!("{}", first));
    println!("{}", format!("{:?}", rest));
    println!("clone_collection_cloning_lock_demo: pass");
}
