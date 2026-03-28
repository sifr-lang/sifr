fn sum_forward(nums: &Vec<i64>) -> i64 {
    let n: i64 = nums.len() as i64;
    let mut total: i64 = 0 as i64;
    for i in 0 as i64..n {
        total = total + nums[i as usize];
    }
    return total;
}

fn sum_reverse(nums: &Vec<i64>) -> i64 {
    let n: i64 = nums.len() as i64;
    let mut total: i64 = 0 as i64;
    for i in (-(1 as i64) + (1 as i64)..(n - (1 as i64)) + (1 as i64)).rev() {
        total = total + nums[i as usize];
    }
    return total;
}

fn sum_reverse_while(nums: &Vec<i64>) -> i64 {
    let n: i64 = nums.len() as i64;
    let mut i: i64 = n - (1 as i64);
    let mut total: i64 = 0 as i64;
    while i >= (0 as i64) {
        total = total + nums[i as usize];
        i -= 1 as i64;
    }
    return total;
}

fn append_growth_product(nums: &Vec<i64>) -> i64 {
    let n: i64 = nums.len() as i64;
    let mut weights: Vec<i64> = vec![];
    for i in 0 as i64..n {
        weights.push(1 as i64);
    }
    let mut i: i64 = n - (1 as i64);
    let mut product: i64 = 1 as i64;
    while i >= (0 as i64) {
        product = product * weights[i as usize];
        i -= 1 as i64;
    }
    return product;
}

fn main() {
    assert!(sum_forward(&vec![4 as i64, 5 as i64, 6 as i64]) == (15 as i64));
    assert!(sum_reverse(&vec![1 as i64, 2 as i64, 3 as i64, 4 as i64]) == (10 as i64));
    assert!(sum_reverse(&vec![]) == (0 as i64));
    assert!(sum_reverse_while(&vec![1 as i64, 2 as i64, 3 as i64, 4 as i64]) == (10 as i64));
    assert!(sum_reverse_while(&vec![]) == (0 as i64));
    assert!(append_growth_product(&vec![2 as i64, 3 as i64, 4 as i64]) == (1 as i64));
    assert!(append_growth_product(&vec![]) == (1 as i64));
}
