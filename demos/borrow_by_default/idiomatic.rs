fn get_length(items: &Vec<i64>) -> i64 {
    return items.len() as i64;
}

fn get_first_char(s: &String) -> String {
    let result: Option<String> = {
        let __sifr_index_str = &s;
        let __sifr_index_i = 0 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_str
            .chars()
            .nth(__sifr_index_norm)
            .map(|c| c.to_string())
    };
    if let Some(result) = result {
        return result;
    }
    return "".to_string();
}

fn consume_and_count(items: Vec<i64>) -> i64 {
    return items.len() as i64;
}

fn add(x: i64, y: i64) -> i64 {
    return x + y;
}

fn is_positive(n: f64) -> bool {
    return n > (0.0 as f64);
}

fn process_data(data: &Vec<i64>) -> i64 {
    let mut total: i64 = 0 as i64;
    for item in data.iter().copied() {
        total = total + item;
    }
    return total;
}

fn sum_multiple_times(items: &Vec<i64>, times: i64) -> i64 {
    let mut total: i64 = 0 as i64;
    for i in 0 as i64..times {
        total = total + get_length(items);
    }
    return total;
}

fn apply_and_return(f: impl Fn(&Vec<i64>) -> i64, items: &Vec<i64>) -> i64 {
    return f(items);
}

fn compute_sum(nums: &Vec<i64>) -> i64 {
    let mut total: i64 = 0 as i64;
    for n in nums.iter().copied() {
        total = total + n;
    }
    return total;
}

fn main() {
    let my_list: Vec<i64> = vec![10 as i64, 20 as i64, 30 as i64];
    let length: i64 = get_length(&my_list);
    println!("{}", length);
    println!("{:?}", my_list);
    let greeting: String = "Hello, Sifr!".to_string();
    let first: String = get_first_char(&greeting);
    println!("{}", first);
    println!("{}", greeting);
    let owned_list: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64, 4 as i64, 5 as i64];
    let count: i64 = consume_and_count(owned_list);
    println!("{}", count);
    let result: i64 = add(10 as i64, 20 as i64);
    println!("{}", result);
    let pi: f64 = 3.14 as f64;
    println!("{}", is_positive(pi));
    println!("{}", pi);
    let data: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64, 4 as i64, 5 as i64];
    let total: i64 = process_data(&data);
    println!("{}", total);
    println!("{:?}", data);
    let items: Vec<i64> = vec![10 as i64, 20 as i64, 30 as i64];
    let loop_total: i64 = sum_multiple_times(&items, 3 as i64);
    println!("{}", loop_total);
    println!("{:?}", items);
    let nums: Vec<i64> = vec![5 as i64, 10 as i64, 15 as i64];
    let sum_result: i64 = apply_and_return(compute_sum, &nums);
    println!("{}", sum_result);
    println!("{:?}", nums);
}
