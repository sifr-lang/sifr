use std::num::ParseIntError;

#[derive(Debug, Clone)]
struct StatisticsError {
    message: String,
}

impl StatisticsError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
struct CycleError {
    message: String,
}

impl CycleError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

fn compute_mean(data: &[f64]) -> Result<f64, StatisticsError> {
    if data.is_empty() {
        return Err(StatisticsError::new("cannot compute mean of empty dataset"));
    }
    Ok(data.iter().sum::<f64>() / data.len() as f64)
}

fn topo_sort(has_cycle: bool) -> Result<i64, CycleError> {
    if has_cycle {
        return Err(CycleError::new("graph contains a cycle"));
    }
    Ok(42)
}

fn parse_int(text: &str) -> Result<i64, ParseIntError> {
    text.parse()
}

fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        format!("{value}")
    }
}

fn main() {
    match compute_mean(&[1.0, 2.0, 3.0]) {
        Ok(mean) => println!("mean = {}", format_number(mean)),
        Err(err) => println!("stats error: {}", err.message),
    }

    let empty: [f64; 0] = [];
    if let Err(err) = compute_mean(&empty) {
        println!("caught StatisticsError: {}", err.message);
    }

    match topo_sort(false) {
        Ok(order) => println!("topo sort result = {order}"),
        Err(err) => println!("cycle error: {}", err.message),
    }

    if let Err(err) = topo_sort(true) {
        println!("caught CycleError: {}", err.message);
    }

    match parse_int("not_a_number") {
        Err(err) => println!("caught ParseError: {err}"),
        Ok(_) => {
            if let Err(stats_err) = compute_mean(&empty) {
                println!("caught StatisticsError: {}", stats_err.message);
            }
        }
    }

    println!("all module-specific error types work correctly");
}
