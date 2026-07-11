#[must_use]
pub fn uuid4() -> String {
    uuid::Uuid::new_v4().hyphenated().to_string()
}

#[must_use]
pub fn uuid3_text(namespace: &str, name: &str) -> String {
    name_based_uuid(namespace, name, uuid::Uuid::new_v3)
}

#[must_use]
pub fn uuid5_text(namespace: &str, name: &str) -> String {
    name_based_uuid(namespace, name, uuid::Uuid::new_v5)
}

fn name_based_uuid(
    namespace: &str,
    name: &str,
    build: fn(&uuid::Uuid, &[u8]) -> uuid::Uuid,
) -> String {
    let namespace = uuid::Uuid::parse_str(namespace).unwrap_or_else(|_| uuid::Uuid::nil());
    build(&namespace, name.as_bytes()).hyphenated().to_string()
}
