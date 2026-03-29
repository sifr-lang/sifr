fn sum_forward(nums: &[i64]) -> i64 {
    nums.iter().sum()
}

fn sum_reverse(nums: &[i64]) -> i64 {
    nums.iter().rev().sum()
}

fn sum_reverse_while(nums: &[i64]) -> i64 {
    let mut total = 0;
    let mut i = nums.len() as isize - 1;
    while i >= 0 {
        total += nums[i as usize];
        i -= 1;
    }
    total
}

fn append_growth_product(nums: &[i64]) -> i64 {
    let weights = vec![1; nums.len()];
    let mut product = 1;
    let mut i = weights.len() as isize - 1;
    while i >= 0 {
        product *= weights[i as usize];
        i -= 1;
    }
    product
}

fn main() {
    assert_eq!(sum_forward(&[4, 5, 6]), 15);
    assert_eq!(sum_reverse(&[1, 2, 3, 4]), 10);
    assert_eq!(sum_reverse(&[]), 0);
    assert_eq!(sum_reverse_while(&[1, 2, 3, 4]), 10);
    assert_eq!(sum_reverse_while(&[]), 0);
    assert_eq!(append_growth_product(&[2, 3, 4]), 1);
    assert_eq!(append_growth_product(&[]), 1);
}
