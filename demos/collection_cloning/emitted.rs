// src/main.rs
mod sifr_generated_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ValueError {
        #[must_use]
        pub const fn new(message: String) -> Self {
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
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::ValueError;
fn double(n: SifrInt) -> SifrInt {
    ::std::ops::Mul::mul(n, &SifrInt::from_i64(2))
}
fn is_even(n: &SifrInt) -> bool {
    n.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == SifrInt::from_i64(0)
}
#[expect(
    clippy::assertions_on_constants,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn main() {
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
    ];
    let mapped: Vec<SifrInt> = Box::new(nums.iter().cloned().map(double)).collect::<Vec<_>>();
    let filtered: Vec<SifrInt> = Box::new(
        nums.iter()
            .filter(|&sifr_generated_filter_item| {
                let sifr_generated_filter_value = sifr_generated_filter_item.clone();
                is_even(&sifr_generated_filter_value)
            })
            .cloned(),
    )
    .collect::<Vec<_>>();
    let mut first: SifrInt = SifrInt::from_i64(0);
    let mut rest: Vec<SifrInt> = Vec::new();
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let sifr_generated_unpack_source = &nums;
        let [sifr_generated_before_0, sifr_generated_star @ ..] =
            sifr_generated_unpack_source.as_slice()
        else {
            return Err(ValueError::new("not enough values to unpack".to_string()));
        };
        first.clone_from(sifr_generated_before_0);
        rest = sifr_generated_star.to_vec();
        Ok(())
    })();
    if let Err(_try_err) = sifr_generated_try_res {
        assert!(false);
    }
    println!("{mapped:?}");
    println!("{filtered:?}");
    println!("{first}");
    println!("{rest:?}");
    println!("clone_collection_cloning_lock_demo: pass");
}
