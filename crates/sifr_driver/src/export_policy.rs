pub(crate) fn should_export_callable(module_name: &str, callable_name: &str) -> bool {
    sifr_type_system::should_export_stdlib_declaration(module_name, callable_name)
}
