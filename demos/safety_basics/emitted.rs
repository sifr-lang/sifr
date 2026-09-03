// src/main.rs
mod sifr_generated_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ParseError {
        pub message: String,
    }
    impl ::std::fmt::Display for ParseError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ParseError {}
}
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::ParseError;
fn base64_encode(s: &str) -> String {
    ::sifr_stdlib::base64::base64_encode(s)
}
fn b64encode(s: &str) -> String {
    base64_encode(s)
}
fn assert_vector_eq(actual: &[String], expected: &[String]) {
    assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from(actual.len()) {
        assert_eq!(
            {
                let sifr_generated_condition_list = &actual;
                let sifr_generated_condition_index = i.clone();
                let sifr_generated_condition_normalized = sifr_generated_condition_index
                    .normalize_index_or_len(sifr_generated_condition_list.len());
                sifr_generated_condition_list
                    .get(sifr_generated_condition_normalized)
                    .cloned()
            },
            {
                let sifr_generated_condition_list = &expected;
                let sifr_generated_condition_index = i.clone();
                let sifr_generated_condition_normalized = sifr_generated_condition_index
                    .normalize_index_or_len(sifr_generated_condition_list.len());
                sifr_generated_condition_list
                    .get(sifr_generated_condition_normalized)
                    .cloned()
            }
        );
        i = &i + &SifrInt::from_i64(1);
    }
}
fn main() {
    let sifr_generated_try_res: Result<(), ParseError> = (|| {
        let _bad: String = ::sifr_runtime::encoding::decode_text(
            &vec![255_u8],
            &"utf-8".to_string(),
            &"strict".to_string(),
        )
        .map_err(|sifr_generated_message| ParseError {
            message: sifr_generated_message,
        })?;
        println!("{}", false);
        assert_eq!(false.to_string(), "true");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let _e = sifr_generated_try_err.clone();
        println!("{}", true);
        assert_eq!(true.to_string(), "true");
    }
    let inputs: Vec<String> = vec![
        String::new(),
        "f".to_string(),
        "fo".to_string(),
        "foo".to_string(),
    ];
    let expected: Vec<String> = vec![
        String::new(),
        "Zg==".to_string(),
        "Zm8=".to_string(),
        "Zm9v".to_string(),
    ];
    let mut actual: Vec<String> = Vec::new();
    for s in inputs.iter().cloned() {
        actual.push(b64encode(&s));
    }
    assert_vector_eq(&actual, &expected);
    println!("{}", true);
}
