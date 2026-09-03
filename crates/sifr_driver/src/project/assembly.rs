use super::rust_module_layout::top_level_module_declarations;
use std::collections::HashMap;

pub(crate) fn ordered_non_main_module_names(
    compile_order: &[String],
    rust_files: &HashMap<String, String>,
) -> Vec<String> {
    compile_order
        .iter()
        .filter(|module_name| module_name.as_str() != "main")
        .filter(|module_name| rust_files.contains_key(module_name.as_str()))
        .cloned()
        .collect()
}

pub(crate) fn assemble_project_main_rs(
    compile_order: &[String],
    rust_files: &HashMap<String, String>,
) -> String {
    let mut main_rs = String::new();
    let ordered_non_main = ordered_non_main_module_names(compile_order, rust_files);
    for module_name in top_level_module_declarations(&ordered_non_main) {
        main_rs.push_str("pub mod ");
        main_rs.push_str(&module_name);
        main_rs.push_str(";\n");
    }
    if !ordered_non_main.is_empty() && rust_files.contains_key("main") {
        main_rs.push('\n');
    }
    if let Some(main_code) = rust_files.get("main") {
        main_rs.push_str(main_code);
    }
    main_rs
}
