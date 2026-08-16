
#[allow(dead_code)]
fn __sifr_return_slot_handler<'call>(
    handler: &'call sifr_runtime::interop::structural::SlotHandler<'call>,
) -> &'static sifr_runtime::interop::structural::SlotHandler<'static> {
    let escaped: &'static sifr_runtime::interop::structural::SlotHandler<'static> = handler;
    escaped
}
