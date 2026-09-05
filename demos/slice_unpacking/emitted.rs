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
use ::std::collections::HashMap;
pub use sifr_generated_project_nominals::ValueError;
static SIFR_GENERATED_SIFR_HOISTED_DICT_0: ::std::sync::LazyLock<HashMap<String, SifrInt>> =
    ::std::sync::LazyLock::new(|| {
        HashMap::from([
            ("x".to_string(), SifrInt::from_i64(11)),
            ("y".to_string(), SifrInt::from_i64(22)),
        ])
    });
fn main() {
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(3),
        SifrInt::from_i64(6),
        SifrInt::from_i64(9),
        SifrInt::from_i64(12),
    ];
    println!(
        "{}",
        {
            let sifr_generated_index_list = &nums;
            let sifr_generated_index_i = SifrInt::from_i64(0);
            let sifr_generated_index_norm =
                sifr_generated_index_i.normalize_index_or_len(sifr_generated_index_list.len());
            sifr_generated_index_list
                .get(sifr_generated_index_norm)
                .cloned()
        }
        .map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
    println!(
        "{}",
        {
            let sifr_generated_index_list = &nums;
            let sifr_generated_index_i = SifrInt::from_i64(99);
            let sifr_generated_index_norm =
                sifr_generated_index_i.normalize_index_or_len(sifr_generated_index_list.len());
            sifr_generated_index_list
                .get(sifr_generated_index_norm)
                .cloned()
        }
        .map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
    let scores = &*SIFR_GENERATED_SIFR_HOISTED_DICT_0;
    println!(
        "{}",
        scores
            .get("x")
            .cloned()
            .unwrap_or_else(|| "None".to_string(),)
    );
    println!(
        "{}",
        scores
            .get("z")
            .cloned()
            .unwrap_or_else(|| "None".to_string(),)
    );
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let sifr_generated_unpack_source = &nums;
        let [
            sifr_generated_before_0,
            sifr_generated_star @ ..,
            sifr_generated_after_0,
        ] = sifr_generated_unpack_source.as_slice()
        else {
            return Err(ValueError::new("not enough values to unpack".to_string()));
        };
        let a = sifr_generated_before_0;
        let mid = sifr_generated_star.to_vec();
        let b = sifr_generated_after_0;
        println!("{a}");
        println!("{mid:?}");
        println!("{b}");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let error = sifr_generated_try_err;
        println!("{}", error.message);
    }
    println!("{:?}", {
        let sifr_generated_v_5f76 = &nums;
        let sifr_generated_len = sifr_generated_v_5f76.len();
        ::sifr_runtime::SifrSliceIndices::new_known_nonzero(
            sifr_generated_len,
            None,
            None,
            &SifrInt::from_i64(2),
        )
        .filter_map(|sifr_generated_i| sifr_generated_v_5f76.get(sifr_generated_i).cloned())
        .collect::<Vec<_>>()
    });
    println!("{}", SifrInt::from(nums.len()));
    println!("clone_slice_unpacking_slice_unpack_demo: pass");
}
