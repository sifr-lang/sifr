use sifr_runtime::interop::CallScopedCallbackBridge;

fn defer_callback<'call>(
    callback: CallScopedCallbackBridge<'call, (String,), ()>,
) -> Box<dyn FnOnce() + 'static> {
    Box::new(move || {
        callback.call(("after-return".to_string(),));
    })
}

pub fn store_for_later(
    callback: CallScopedCallbackBridge<'_, (String,), ()>,
) -> Result<(), String> {
    let _deferred = defer_callback(callback);
    Ok(())
}
