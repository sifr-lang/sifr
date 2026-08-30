use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use wit_component::ComponentEncoder;

fn main() -> Result<(), Box<dyn Error>> {
    let mut arguments = env::args_os().skip(1);
    let input = arguments
        .next()
        .map(PathBuf::from)
        .ok_or("missing core WebAssembly input path")?;
    let output = arguments
        .next()
        .map(PathBuf::from)
        .ok_or("missing component output path")?;
    if arguments.next().is_some() {
        return Err("componentize accepts exactly two paths".into());
    }
    let module = fs::read(input)?;
    let component = ComponentEncoder::default().module(&module)?.encode()?;
    fs::write(output, component)?;
    Ok(())
}
