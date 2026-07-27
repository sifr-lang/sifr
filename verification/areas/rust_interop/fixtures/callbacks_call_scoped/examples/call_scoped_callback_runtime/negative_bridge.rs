use std::cell::RefCell;

use sifr_runtime::interop::CallScopedCallbackBridge;

thread_local! {
    static STORED: RefCell<
        Option<CallScopedCallbackBridge<'static, (String,), ()>>
    > = const { RefCell::new(None) };
}

pub fn store_for_later<'call>(
    callback: CallScopedCallbackBridge<'call, (String,), ()>,
) -> Result<(), String> {
    STORED.with(|slot| {
        *slot.borrow_mut() = Some(callback);
    });
    Ok(())
}
