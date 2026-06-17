use csv::{ReaderBuilder, WriterBuilder};
use std::fs;
use std::io;

type IOError = io::Error;

fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(actual, expected);
}

fn parse_row(row: &str) -> Vec<String> {
    parse_csv(row).into_iter().next().unwrap_or_default()
}

fn parse_csv(text: &str) -> Vec<Vec<String>> {
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(text.as_bytes());

    reader
        .records()
        .filter_map(Result::ok)
        .map(|record| record.iter().map(str::to_string).collect())
        .collect()
}

fn format_csv(rows: &[Vec<String>]) -> String {
    let mut writer = WriterBuilder::new()
        .has_headers(false)
        .from_writer(Vec::new());

    for row in rows {
        let _ = writer.write_record(row);
    }

    let bytes = match writer.into_inner() {
        Ok(bytes) => bytes,
        Err(_) => return String::new(),
    };

    String::from_utf8(bytes)
        .unwrap_or_default()
        .trim_end_matches('\n')
        .trim_end_matches('\r')
        .to_string()
}

struct CsvReader {
    rows: Vec<Vec<String>>,
}

impl CsvReader {
    fn new(text: &str) -> Self {
        Self {
            rows: parse_csv(text),
        }
    }

    fn rows(&self) -> &[Vec<String>] {
        &self.rows
    }
}

#[derive(Default)]
struct CsvWriter {
    rows: Vec<Vec<String>>,
}

impl CsvWriter {
    fn writerow(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }

    fn writerows(&mut self, rows: Vec<Vec<String>>) {
        self.rows.extend(rows);
    }

    fn getvalue(&self) -> String {
        format_csv(&self.rows)
    }
}

fn reader_from_path(path: &str) -> Result<CsvReader, IOError> {
    Ok(CsvReader::new(&fs::read_to_string(path)?))
}

fn writer_to_path(path: &str, rows: Vec<Vec<String>>) -> Result<(), IOError> {
    let mut writer = CsvWriter::default();
    writer.writerows(rows);
    fs::write(path, writer.getvalue())
}

fn collect_parse_actual() -> Vec<bool> {
    vec![
        parse_row("a,b,c") == ["a", "b", "c"].map(str::to_string),
        format_csv(&[
            vec!["1".to_string(), "2".to_string()],
            vec!["3".to_string(), "4".to_string()],
        ]) == "1,2\n3,4",
    ]
}

fn collect_object_and_file_actual() -> Vec<bool> {
    let reader = CsvReader::new("name,age\nalice,30");
    let rows_ok = reader.rows()
        == [
            vec!["name".to_string(), "age".to_string()],
            vec!["alice".to_string(), "30".to_string()],
        ];

    let mut writer = CsvWriter::default();
    writer.writerow(vec!["alice".to_string(), "30".to_string()]);
    let writer_ok = writer.getvalue() == "alice,30";

    let path = "/tmp/sifr_csv_csv_demo.csv";
    let csv_file_ok = writer_to_path(
        path,
        vec![
            vec!["h1".to_string(), "h2".to_string()],
            vec!["v1".to_string(), "v2".to_string()],
        ],
    )
    .and_then(|_| reader_from_path(path))
    .map(|reader| {
        reader.rows()
            == [
                vec!["h1".to_string(), "h2".to_string()],
                vec!["v1".to_string(), "v2".to_string()],
            ]
    })
    .unwrap_or(false);

    let missing_rejected = reader_from_path("/tmp/sifr_csv_csv_demo_missing.csv").is_err();

    vec![rows_ok, writer_ok, csv_file_ok, missing_rejected]
}

fn main() {
    let mut actual = Vec::new();
    actual.extend(collect_parse_actual());
    actual.extend(collect_object_and_file_actual());

    assert_bool_vector_eq(&actual, &[true, true, true, true, true, true]);
    println!("csv csv parity demo: pass");
}
