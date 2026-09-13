use super::*;

#[test]
fn file_patterns_mutate_registered_handles_without_consuming_them()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = std::env::temp_dir().join(format!(
        "sifr-file-patterns-{}-{}",
        std::process::id(),
        __sifr_next_file_handle_id()
    ));
    std::fs::create_dir(&directory)?;
    let path = directory.join("contents");
    let name = path.to_string_lossy().into_owned();

    let mut writer = open(&name, &"w".to_string())?;
    writer.write(&"alpha\n".to_string())?;
    writer.write(&"beta\ngamma\n".to_string())?;
    assert!(!writer.closed());
    writer.close();
    assert!(writer.closed());
    assert!(writer.write(&"closed".to_string()).is_err());

    let mut reader = open(&name, &"r".to_string())?;
    assert_eq!(reader.readline()?, Some("alpha".to_string()));
    assert_eq!(reader.read()?, "beta\ngamma\n");
    assert_eq!(reader.read()?, "");
    assert!(!reader.closed());
    reader.close();
    assert!(reader.read().is_err());

    let mut lines = open(&name, &"r".to_string())?;
    assert_eq!(lines.readlines()?, vec!["alpha", "beta", "gamma"]);
    assert!(lines.readlines()?.is_empty());
    lines.close();

    let mut byte_writer = open(&name, &"wb".to_string())?;
    byte_writer.write_bytes(&[0, 1])?;
    byte_writer.write_bytes(&[2, 255])?;
    byte_writer.close();
    let mut byte_reader = open(&name, &"rb".to_string())?;
    assert_eq!(byte_reader.read_bytes()?, vec![0, 1, 2, 255]);
    assert!(byte_reader.read_bytes()?.is_empty());
    byte_reader.close();
    assert!(byte_reader.read_bytes().is_err());

    let mut binary_writer = open_binary(&name, &"wb".to_string())?;
    binary_writer.write_bytes(&[3, 4])?;
    binary_writer.write_bytes(&[5, 254])?;
    assert!(!binary_writer.closed());
    binary_writer.close();
    assert!(binary_writer.write_bytes(&[6]).is_err());
    let mut binary_reader = open_binary(&name, &"rb".to_string())?;
    assert_eq!(binary_reader.read_bytes(None)?, vec![3, 4, 5, 254]);
    assert!(binary_reader.read_bytes(None)?.is_empty());
    assert!(!binary_reader.closed());
    binary_reader.close();
    assert!(binary_reader.read_bytes(None).is_err());

    std::fs::remove_file(path)?;
    std::fs::remove_dir(directory)?;
    Ok(())
}
