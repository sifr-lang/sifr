use std::collections::{HashMap, HashSet};

use quote::ToTokens;
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

include!(
    "../../../../../sifr_codegen/src/generated_rust_canonicalizer/syntax_cleanup/idiom_cleanup/clippy_cleanup/expression_cleanup.rs"
);
include!(
    "../../../../../sifr_codegen/src/generated_rust_canonicalizer/syntax_cleanup/idiom_cleanup/clippy_cleanup/residual_literal_cleanup.rs"
);
include!(
    "../../../../../sifr_codegen/src/generated_rust_canonicalizer/syntax_cleanup/idiom_cleanup/clippy_cleanup/structural_expression_cleanup.rs"
);
include!(
    "../../../../../sifr_codegen/src/generated_rust_canonicalizer/syntax_cleanup/idiom_cleanup/clippy_cleanup/typed_value_planning.rs"
);
include!(
    "../../../../../sifr_codegen/src/generated_rust_canonicalizer/syntax_cleanup/idiom_cleanup/clippy_cleanup/sifr_int_operation_cleanup.rs"
);
include!(
    "../../../../../sifr_codegen/src/generated_rust_canonicalizer/syntax_cleanup/idiom_cleanup/clippy_cleanup/borrowed_clone_cleanup.rs"
);
include!(
    "../../../../../sifr_codegen/src/generated_rust_canonicalizer/syntax_cleanup/idiom_cleanup/clippy_cleanup/typed_value_cleanup.rs"
);
include!(
    "../../../../../sifr_codegen/src/generated_rust_canonicalizer/syntax_cleanup/idiom_cleanup/clippy_cleanup/copy_iterator_cleanup.rs"
);
include!(
    "../../../../../sifr_codegen/src/generated_rust_canonicalizer/syntax_cleanup/idiom_cleanup/clippy_cleanup/residual_call_cleanup.rs"
);
include!(
    "../../../../../sifr_codegen/src/generated_rust_canonicalizer/syntax_cleanup/idiom_cleanup/clippy_cleanup/liveness_cleanup.rs"
);
include!(
    "../../../../../sifr_codegen/src/generated_rust_canonicalizer/syntax_cleanup/idiom_cleanup/clippy_cleanup/write_only_collection_cleanup.rs"
);
include!(
    "../../../../../sifr_codegen/src/generated_rust_canonicalizer/syntax_cleanup/idiom_cleanup/clippy_cleanup/local_type_expectation.rs"
);
include!(
    "../../../../../sifr_codegen/src/generated_rust_canonicalizer/syntax_cleanup/idiom_cleanup/clippy_cleanup/generated_index_cleanup.rs"
);
include!(
    "../../../../../sifr_codegen/src/generated_rust_canonicalizer/syntax_cleanup/idiom_cleanup/clippy_cleanup/loop_cleanup.rs"
);

include!(
    "../../../../../sifr_codegen/src/generated_rust_canonicalizer/syntax_cleanup/idiom_cleanup/clippy_cleanup/typed_string_concat.rs"
);
