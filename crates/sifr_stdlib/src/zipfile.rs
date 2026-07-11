use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use zip_8_6::{write::SimpleFileOptions, ZipArchive, ZipWriter};

pub fn zip_create(path: &str) -> Result<(), std::io::Error> {
    let file = File::create(path)?;
    ZipWriter::new(file).finish().map_err(zip_error)?;
    Ok(())
}

pub fn zip_add_file(zip_path: &str, name: &str, content: &str) -> Result<(), std::io::Error> {
    zip_add_payload(zip_path, name, content.as_bytes())
}

pub fn zip_add_file_bytes(
    zip_path: &str,
    name: &str,
    content: &[u8],
) -> Result<(), std::io::Error> {
    zip_add_payload(zip_path, name, content)
}

pub fn zip_read_file(zip_path: &str, name: &str) -> Result<String, std::io::Error> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file).map_err(zip_error)?;
    let mut file = archive.by_name(name).map_err(zip_error)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

pub fn zip_read_file_bytes(zip_path: &str, name: &str) -> Result<Vec<u8>, std::io::Error> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file).map_err(zip_error)?;
    let mut file = archive.by_name(name).map_err(zip_error)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    Ok(content)
}

pub fn zip_namelist(zip_path: &str) -> Result<Vec<String>, std::io::Error> {
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file).map_err(zip_error)?;
    let mut names = Vec::new();
    for index in 0..archive.len() {
        if let Ok(file) = archive.by_index(index) {
            names.push(file.name().to_string());
        }
    }
    Ok(names)
}

fn zip_add_payload(zip_path: &str, name: &str, content: &[u8]) -> Result<(), std::io::Error> {
    let file = OpenOptions::new().read(true).write(true).open(zip_path)?;
    let mut archive = ZipWriter::new_append(file).map_err(zip_error)?;
    archive
        .start_file(name.to_string(), SimpleFileOptions::default())
        .map_err(zip_error)?;
    archive.write_all(content)?;
    archive.finish().map_err(zip_error)?;
    Ok(())
}

fn zip_error(error: zip_8_6::result::ZipError) -> std::io::Error {
    std::io::Error::other(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        zip_add_file, zip_add_file_bytes, zip_create, zip_namelist, zip_read_file,
        zip_read_file_bytes,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn zipfile_adapter_round_trips_text_and_names() {
        let archive = temp_archive_path("roundtrip");
        zip_create(&archive).expect("zip should be created");
        assert!(zip_namelist(&archive)
            .expect("empty archive should be readable")
            .is_empty());
        zip_add_file(&archive, "hello.txt", "hello zip").expect("text should be written");
        zip_add_file_bytes(&archive, "bytes.bin", &[1, 2, 3]).expect("bytes should be written");

        let names = zip_namelist(&archive).expect("names should read");
        assert!(names.iter().any(|name| name == "hello.txt"));
        assert!(names.iter().any(|name| name == "bytes.bin"));
        assert_eq!(
            zip_read_file(&archive, "hello.txt").expect("text should read"),
            "hello zip"
        );
        assert_eq!(
            zip_read_file_bytes(&archive, "bytes.bin").expect("bytes should read"),
            vec![1, 2, 3]
        );

        let _ = std::fs::remove_file(archive);
    }

    #[test]
    fn zipfile_adapter_reports_missing_file() {
        let err =
            zip_namelist("__sifr_missing_archive__.zip").expect_err("missing archive should fail");
        assert!(!err.to_string().is_empty());
    }

    fn temp_archive_path(label: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        std::env::temp_dir()
            .join(format!("sifr_zipfile_{label}_{nanos}.zip"))
            .display()
            .to_string()
    }
}
