use super::rust_interop_probe::zero_copy_type_probe_source;

#[test]
fn zero_copy_type_probe_emits_each_declared_thread_obligation() {
    for (obligations, send, sync) in [
        ((false, false), false, false),
        ((true, false), true, false),
        ((false, true), false, true),
        ((true, true), true, true),
    ] {
        let source = zero_copy_type_probe_source(obligations, "bridge::views::View");

        assert!(source.contains("type __SifrView = bridge::views::View;"));
        assert_eq!(source.contains("__sifr_assert_send::<__SifrView>();"), send);
        assert_eq!(source.contains("__sifr_assert_sync::<__SifrView>();"), sync);
    }
}
