#![no_main]

use libfuzzer_sys::fuzz_target;
use sifr_driver::guided_project_graph::{project_tree, replay};

fuzz_target!(|data: &[u8]| {
    // Export is used only for one-shot post-minimization replay. Sustained fuzz
    // iterations never create fixtures; they overlay the checked-in one.
    if let Ok(output) = std::env::var("SIFR_FUZZ_PROJECT_TREE_EXPORT_DIR") {
        if let Some(tree) = project_tree(data) {
            let root = std::path::Path::new(&output);
            let result = std::fs::create_dir_all(root)
                .and_then(|()| std::fs::write(root.join("sifr.toml"), tree.manifest))
                .and_then(|()| {
                    for (name, source) in tree.files {
                        std::fs::write(root.join(name), source)?;
                    }
                    Ok(())
                });
            if let Err(error) = result {
                eprintln!("PROJECT_TREE_EXPORT_ERROR: {error}");
                return;
            }
        }
    }
    let _ = replay(data);
});
