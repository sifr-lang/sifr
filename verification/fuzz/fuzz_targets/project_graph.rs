#![no_main]

use libfuzzer_sys::fuzz_target;
use sifr_driver::guided_project_graph::{project_tree, replay};
use sifr_frontend::DiskSourceProvider;

fuzz_target!(|data: &[u8]| {
    // Export is used only for one-shot post-minimization replay. Sustained fuzz
    // iterations use source overlays and never write the checked-in fixture.
    if let Ok(output) = std::env::var("SIFR_FUZZ_PROJECT_TREE_EXPORT_DIR") {
        if let Some(tree) = project_tree(data) {
            if let Err(error) = tree.write_to(std::path::Path::new(&output)) {
                eprintln!("PROJECT_TREE_EXPORT_ERROR: {error}");
                return;
            }
        }
    }
    let _ = replay(data, DiskSourceProvider::new());
});
