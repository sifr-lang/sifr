use crate::{RustItem, RustStmt};

pub(crate) fn scope_async_main_cancellation(items: &mut [RustItem]) {
    for item in items {
        let RustItem::Fn {
            name,
            is_async: true,
            body,
            ..
        } = item
        else {
            continue;
        };
        if name != "main" {
            continue;
        }
        let original = std::mem::take(body);
        let rendered = crate::render_stmts(&original);
        body.push(RustStmt::compiler_fragment(format!(
            "let __sifr_root_cancellation = ::sifr_runtime::cancellation::CancellationCarrier::new(); return __SIFR_TASK_CANCELLATION.scope(__sifr_root_cancellation, async move {{ {rendered} }}).await;"
        )));
        return;
    }
}
