
#[allow(dead_code)]
fn __sifr_mutate_shared_context(context: &mut SharedContext<'_, String>) {
    context.get().push_str("forbidden");
}
