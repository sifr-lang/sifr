use sifr_ir::HirClass;
use sifr_type_system::class_rust_name;

pub(super) fn target(class: &HirClass) -> String {
    let rust_name = class_rust_name(class.identity.as_deref(), &class.name);
    if class.type_params.is_empty() {
        rust_name
    } else {
        format!("{rust_name}<{}>", class.type_params.join(", "))
    }
}
