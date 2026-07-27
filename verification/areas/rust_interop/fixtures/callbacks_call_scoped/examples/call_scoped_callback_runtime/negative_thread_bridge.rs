use sifr_runtime::interop::CallScopedCallbackBridge;

pub fn store_for_later(
    callback: CallScopedCallbackBridge<'_, (String,), ()>,
) -> Result<(), String> {
    let deferred = std::thread::spawn(move || {
        callback.call(("after-return".to_string(),));
    });
    deferred
        .join()
        .map_err(|_| "callback thread panicked".to_string())?;
    Ok(())
}
