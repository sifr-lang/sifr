fn second_or_zero(values: &[i64]) -> i64 {
    if values.len() < 2 {
        0
    } else {
        values[1]
    }
}

fn neighbor_min_cost(mut cost: Vec<i64>) -> i64 {
    if cost.len() < 2 {
        return 0;
    }

    for index in (0..=cost.len() - 3).rev() {
        cost[index] += std::cmp::min(cost[index + 1], cost[index + 2]);
    }

    std::cmp::min(cost[0], cost[1])
}

fn main() {
    assert_eq!(second_or_zero(&[8, 13]), 13);
    assert_eq!(second_or_zero(&[8]), 0);
    assert_eq!(neighbor_min_cost(vec![10, 15, 20]), 15);
}
