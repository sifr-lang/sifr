fn render() -> String {
    match toml::from_str::<toml::Value>("name = \"five\"\nvalue = 5") {
        Ok(parsed) if parsed.get("name").is_some() && parsed.get("value").is_some() => {
            "dependency closure demo: pass".to_string()
        }
        Ok(_) => "dependency closure demo: empty".to_string(),
        Err(err) => err.to_string(),
    }
}

fn main() {
    println!("{}", render());
}
