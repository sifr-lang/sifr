// src/main.rs
fn sum_forward(nums: &Vec<i64>) -> i64 {
    let n: i64 = nums.len() as i64;
    let mut total: i64 = 0_i64;
    for i in 0_i64..n {
        total += nums[i as usize];
    }
    total
}

fn sum_reverse(nums: &Vec<i64>) -> i64 {
    let n: i64 = nums.len() as i64;
    let mut total: i64 = 0_i64;
    for i in (-(1_i64) + (1_i64)..(n - (1_i64)) + (1_i64)).rev() {
        total += nums[i as usize];
    }
    total
}

fn sum_reverse_while(nums: &Vec<i64>) -> i64 {
    let n: i64 = nums.len() as i64;
    let mut i: i64 = n - (1_i64);
    let mut total: i64 = 0_i64;
    while i >= (0_i64) {
        total += nums[i as usize];
        i -= 1_i64;
    }
    total
}

fn append_growth_product(nums: &Vec<i64>) -> i64 {
    let n: i64 = nums.len() as i64;
    let mut weights: Vec<i64> = vec![];
    for i in 0_i64..n {
        weights.push(1_i64);
    }
    let mut i: i64 = n - (1_i64);
    let mut product: i64 = 1_i64;
    while i >= (0_i64) {
        product *= weights[i as usize];
        i -= 1_i64;
    }
    product
}

fn main() {
    assert!((sum_forward(&vec![4_i64, 5_i64, 6_i64]) == (15_i64)));
    assert!((sum_reverse(&vec![1_i64, 2_i64, 3_i64, 4_i64]) == (10_i64)));
    assert!((sum_reverse(&vec![]) == (0_i64)));
    assert!((sum_reverse_while(&vec![1_i64, 2_i64, 3_i64, 4_i64]) == (10_i64)));
    assert!((sum_reverse_while(&vec![]) == (0_i64)));
    assert!((append_growth_product(&vec![2_i64, 3_i64, 4_i64]) == (1_i64)));
    assert!((append_growth_product(&vec![]) == (1_i64)));
}
