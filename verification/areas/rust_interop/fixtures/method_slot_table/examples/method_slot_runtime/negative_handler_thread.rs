
#[allow(dead_code)]
fn __sifr_send_slot_handler(
    handler: sifr_runtime::interop::structural::SlotHandler<'_>,
) {
    std::thread::scope(|scope| {
        scope.spawn(move || drop(handler));
    });
}
