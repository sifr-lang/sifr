use super::*;

#[test]
fn mutable_class_field_guard_lowers_sort_checked_read_and_pop_in_order() {
    let generated = generate_rust_from_source(
        r#"
class SeatManager:
    available: list[int]

    def __init__(self):
        self.available = []

    def reserve(mut self) -> int:
        if len(self.available) > 0:
            self.available.sort()
            seat = self.available[0]
            self.available.pop(0)
            if seat is None:
                return -1
            return seat
        return -1
"#,
    );

    assert!(!generated.contains("compile_error!"), "{generated}");
    let guarded_read = generated
        .find("if let Some(__sifr_checked_value_0)")
        .expect("guarded field read");
    let sort = generated
        .find("self.available.sort()")
        .expect("sort on original field");
    let refreshed_read = generated[sort..]
        .find("__sifr_checked_read_collection = &self.available")
        .expect("read refreshed after sort")
        + sort;
    let pop = generated
        .find("self.available.remove(0_usize)")
        .expect("pop on original field");
    assert!(guarded_read < sort && sort < refreshed_read && refreshed_read < pop);
    assert!(!generated.contains("self.available.clone()"), "{generated}");
}
