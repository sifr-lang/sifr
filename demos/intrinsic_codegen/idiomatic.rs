fn pow(x: f64, y: f64) -> f64 {
    x.powf(y)
}

fn main() {
    let base: f64 = 9.0;
    let root = base.sqrt();
    let rounded_down = 3.9_f64.floor() as i64;
    let rounded_up = 3.1_f64.ceil() as i64;
    let powered = pow(2.0, 3.0);
    let rounded = 3.6_f64.round() as i64;
    let angle = 1.0_f64.atan2(1.0);
    let finite = powered.is_finite();

    println!("root = {root}");
    assert_eq!(format!("root = {root}"), "root = 3");
    println!("rounded_down = {rounded_down}");
    assert_eq!(format!("rounded_down = {rounded_down}"), "rounded_down = 3");
    println!("rounded_up = {rounded_up}");
    assert_eq!(format!("rounded_up = {rounded_up}"), "rounded_up = 4");
    println!("powered = {powered}");
    assert_eq!(format!("powered = {powered}"), "powered = 8");
    println!("rounded = {rounded}");
    assert_eq!(format!("rounded = {rounded}"), "rounded = 4");
    println!("angle_positive = {}", angle > 0.0);
    assert_eq!(
        format!("angle_positive = {}", angle > 0.0),
        "angle_positive = true"
    );
    println!("finite = {finite}");
    assert_eq!(format!("finite = {finite}"), "finite = true");
}
