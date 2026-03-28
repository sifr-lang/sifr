fn main() {
    let nums: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64, 4 as i64, 5 as i64];
    let doubled: Vec<i64> =
        Box::new(nums.iter().copied().map(|x| x * (2 as i64))).collect::<Vec<_>>();
    println!("{:?}", doubled);
    let evens: Vec<i64> = Box::new(nums.iter().copied().filter(|__filter_item| {
        let __filter_value = *__filter_item;
        return {
            let x = __filter_value;
            (x % (2 as i64)) == (0 as i64)
        };
    }))
    .collect::<Vec<_>>();
    println!("{:?}", evens);
    let squares: Vec<i64> = {
        let mut __sifr_list_comp = vec![];
        for x in nums.iter().copied() {
            __sifr_list_comp.push(x * x);
        }
        __sifr_list_comp
    };
    println!("{:?}", squares);
    let big_squares: Vec<i64> = {
        let mut __sifr_list_comp = vec![];
        for x in nums.iter().copied() {
            if x > (2 as i64) {
                __sifr_list_comp.push(x * x);
            }
        }
        __sifr_list_comp
    };
    println!("{:?}", big_squares);
    let lo: Option<i64> = (nums).iter().copied().min();
    let hi: Option<i64> = (nums).iter().copied().max();
    if let Some(lo) = lo {
        println!("{}", lo);
    }
    if let Some(hi) = hi {
        println!("{}", hi);
    }
    println!("{}", (nums).iter().copied().sum::<i64>());
    let unsorted: Vec<i64> = vec![5 as i64, 3 as i64, 1 as i64, 4 as i64, 2 as i64];
    println!("{:?}", {
        let mut __sifr_sorted_v = (unsorted).iter().copied().collect::<Vec<_>>();
        __sifr_sorted_v.sort();
        __sifr_sorted_v
    });
    println!(
        "{:?}",
        Box::new((unsorted).iter().copied().rev()).collect::<Vec<_>>()
    );
    let letters: Vec<String> = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    println!(
        "{:?}",
        Box::new(
            (letters)
                .iter()
                .cloned()
                .enumerate()
                .map(|__pair| ((__pair.0 as i64) + 0, __pair.1))
        )
        .collect::<Vec<_>>()
    );
    let names: Vec<String> = vec!["Alice".to_string(), "Bob".to_string()];
    let ages: Vec<i64> = vec![30 as i64, 25 as i64];
    println!(
        "{:?}",
        Box::new(
            (names)
                .iter()
                .cloned()
                .zip((ages).iter().copied())
                .map(|__zip_item| (__zip_item.0, __zip_item.1))
        )
        .collect::<Vec<_>>()
    );
    let bools: Vec<bool> = vec![true, false, true];
    println!("{}", (bools).iter().copied().any(|x| x));
    println!("{}", (bools).iter().copied().all(|x| x));
    println!("{}", (vec![true, true, true]).into_iter().all(|x| x));
}
