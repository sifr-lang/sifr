// src/main.rs
use ::sifr_runtime::SifrInt;
use ::sifr_runtime::SifrRange;
use ::std::collections::HashMap;
use ::std::collections::HashSet;
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    let mut words: Vec<String> = vec!["core".to_string()];
    words.extend(
        "xy".to_string()
            .chars()
            .map(|sifr_generated_char| sifr_generated_char.to_string()),
    );
    println!("{}", format!("{words:?}"));
    let mut mapping: HashMap<String, SifrInt> =
        HashMap::from([("base".to_string(), SifrInt::from_i64(1))]);
    mapping.extend(HashMap::from([("extra".to_string(), SifrInt::from_i64(2))]));
    println!(
        "{}",
        mapping
            .remove(&"missing".to_string())
            .unwrap_or(SifrInt::from_i64(7))
    );
    let mut seen: HashSet<SifrInt> = HashSet::from([SifrInt::from_i64(1)]);
    {
        seen.extend(vec![SifrInt::from_i64(2), SifrInt::from_i64(3)].into_iter());
        seen.extend(SifrRange::new_known_nonzero(
            SifrInt::from_i64(4),
            SifrInt::from_i64(6),
            SifrInt::from_i64(1),
        ));
        ()
    };
    {
        let sifr_generated_other = vec![SifrInt::from_i64(3), SifrInt::from_i64(9)]
            .into_iter()
            .collect::<std::collections::HashSet<_>>();
        seen = seen
            .symmetric_difference(&sifr_generated_other)
            .cloned()
            .collect::<std::collections::HashSet<_>>();
        ()
    };
    println!("{}", seen.contains(&SifrInt::from_i64(9)));
    let pair: (SifrInt, SifrInt, SifrInt) = (
        SifrInt::from_i64(4),
        SifrInt::from_i64(5),
        SifrInt::from_i64(4),
    );
    println!("{}", {
        let mut sifr_generated_count = 0;
        if &pair.0 == &SifrInt::from_i64(4) {
            sifr_generated_count += 1;
        }
        if &pair.1 == &SifrInt::from_i64(4) {
            sifr_generated_count += 1;
        }
        if &pair.2 == &SifrInt::from_i64(4) {
            sifr_generated_count += 1;
        }
        SifrInt::from(sifr_generated_count)
    });
    println!(
        "{}",
        {
            let sifr_generated_start = SifrInt::from_i64(1).clamp_slice_bound(3usize);
            let sifr_generated_stop = 3usize;
            let mut sifr_generated_result = None;
            if sifr_generated_result == None
                && (0usize >= sifr_generated_start && 0usize < sifr_generated_stop)
                && &pair.0 == &SifrInt::from_i64(4)
            {
                sifr_generated_result = Some(SifrInt::from(0usize));
            }
            if sifr_generated_result == None
                && (1usize >= sifr_generated_start && 1usize < sifr_generated_stop)
                && &pair.1 == &SifrInt::from_i64(4)
            {
                sifr_generated_result = Some(SifrInt::from(1usize));
            }
            if sifr_generated_result == None
                && (2usize >= sifr_generated_start && 2usize < sifr_generated_stop)
                && &pair.2 == &SifrInt::from_i64(4)
            {
                sifr_generated_result = Some(SifrInt::from(2usize));
            }
            sifr_generated_result
        }
        .map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
    println!(
        "{}",
        format!(
            "{:?}",
            if &SifrInt::from_i64(1) < &0 {
                "alpha,beta,gamma"
                    .to_string()
                    .split(',')
                    .map(::std::string::ToString::to_string)
                    .collect::<Vec<String>>()
            } else {
                "alpha,beta,gamma"
                    .to_string()
                    .splitn(
                        ::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(1) + 1)),
                        ',',
                    )
                    .map(::std::string::ToString::to_string)
                    .collect::<Vec<String>>()
            }
        )
    );
    println!(
        "{}",
        if &SifrInt::from_i64(2) < &0 {
            "aaaa".to_string().replace('a', "b")
        } else {
            "aaaa".to_string().replacen(
                'a',
                "b",
                ::sifr_runtime::to_usize_proven(&SifrInt::from_i64(2)),
            )
        }
    );
}
