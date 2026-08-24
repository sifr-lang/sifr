from __future__ import annotations

BUFFER_MATRIX_SPECS = {
    "positive": {
        "typed-runtime-matrix": (
            "runtime",
            "python::buffer_ops::tests",
            {
                "crates/sifr_runtime/src/python/buffer_ops/raw.rs",
                "crates/sifr_runtime/src/python/buffer_ops/tests.rs",
                "crates/sifr_runtime/src/python/buffer_ops/release_evidence_tests.rs",
                "crates/sifr_runtime/src/python/buffer_ops/typed_access_evidence_tests.rs",
            },
            {
                "fixed-width and pointer-width signed and unsigned integers",
                "float",
                "native endian format",
                "C/F/strided/negative-stride/indirect layout",
                "bounded read/write/copy_slice",
                "constant-space contiguous admission",
            },
        ),
        "compiler-contract-matrix": (
            "lowering-codegen",
            "python_buffer_contract_tests and python_buffer_codegen_tests",
            {
                "crates/sifr_lowering/src/lower/python_buffer_contract_tests.rs",
                "crates/sifr_codegen/src/python_buffer_codegen_tests.rs",
            },
            {
                "import-root target",
                "bridge target",
                "read-only Self receiver",
                "producer read/write access",
                "any/C/F layout",
                "affine aggregate propagation",
                "non-Send propagation",
            },
        ),
        "compiled-producer-matrix": (
            "native-binary",
            "runner/buffer_examples.py",
            {
                "verification/areas/python_interop/runner/buffer_examples.py",
                "verification/areas/python_interop/fixtures/numpy_buffer/buffer_declaration_codegen_smoke.sifr",
                "verification/areas/python_interop/fixtures/numpy_buffer/buffer_declaration_self.sifr",
                "verification/areas/python_interop/fixtures/numpy_buffer/buffer_declaration_bridge.sifr",
                "verification/areas/python_interop/fixtures/numpy_buffer/buffer_affine_aggregate_codegen.sifr",
                "verification/areas/python_interop/fixtures/numpy_buffer/buffer_declaration_numpy.sifr",
            },
            {
                "builtins.bytearray import-root",
                "opaque mmap Self receiver",
                "package-local bridge producer",
                "real NumPy ndarray exporter",
                "affine record/Option/list/tuple/union/recursive aggregate",
            },
        ),
        "canonical-python-runtime": (
            "runtime-native-binary",
            "buffer-runtime suite on the exact canonical CPython",
            {
                ".github/workflows/local-first-validation.yml",
                "verification/areas/python_interop/pyproject.toml",
                "verification/areas/python_interop/uv.lock",
                "verification/areas/python_interop/runner/buffer_runtime.py",
            },
            {
                "five C-level exact release and pointer tests",
            },
        ),
    },
    "negative": {
        "declaration-shape": (
            "lowering",
            "python_buffer_contract_tests",
            {"crates/sifr_lowering/src/lower/python_buffer_contract_tests.rs"},
            {
                "invalid target",
                "invalid access/layout",
                "unsupported element type",
                "wrong return channel",
                "async declaration",
                "invalid receiver convention",
            },
        ),
        "ownership-and-traits": (
            "lowering-e2e",
            "python_buffer_contract_tests and crates/sifr/tests/e2e/fail/python_buffer_*.sifr",
            {
                "crates/sifr_lowering/src/lower/python_buffer_contract_tests.rs",
                "crates/sifr/tests/e2e/fail/python_buffer_affine_membership.sifr",
                "crates/sifr/tests/e2e/fail/python_buffer_identity_rejected.sifr",
                "crates/sifr/tests/e2e/fail/python_buffer_lambda_capture_rejected.sifr",
                "crates/sifr/tests/e2e/fail/python_buffer_nested_async_generator_capture.sifr",
                "crates/sifr/tests/e2e/fail/python_buffer_nested_function_capture_rejected.sifr",
                "crates/sifr/tests/e2e/fail/python_buffer_walrus_rejected.sifr",
                "crates/sifr/tests/e2e/fail/python_buffer_writable_producer_borrowed_owner.sifr",
                "crates/sifr/tests/e2e/fail/python_buffer_writable_self_owner_alias.sifr",
            },
            {
                "copy/clone/equality/hash/order",
                "use after move",
                "borrowed release",
                "mutable alias",
                "field/index projection",
                "lambda/nested-function/generator capture",
                "walrus alias",
                "task sendability",
                "writable Self owner alias",
                "writable producer borrowed-owner alias",
            },
        ),
        "runtime-validation": (
            "runtime",
            "python::buffer_ops::tests",
            {"crates/sifr_runtime/src/python/buffer_ops/tests.rs"},
            {
                "dtype/format mismatch",
                "item-size mismatch",
                "layout mismatch",
                "readonly write",
                "bounds",
                "non-buffer exporter",
                "double release",
                "use after release",
                "overlapping writable views",
            },
        ),
    },
    "cleanup": {
        "explicit-release": (
            "runtime-native-binary",
            "C-level exporter counter/pointer identity plus compiled bridge shared mutation and post-release resizability",
            {
                "crates/sifr_runtime/src/python/buffer_ops/release_evidence_tests.rs::instrumented_exporter_explicit_release_is_exact_once_and_pointer_identical",
                "verification/areas/python_interop/fixtures/numpy_buffer/buffer_declaration_bridge.sifr",
                "verification/areas/python_interop/fixtures/numpy_buffer/buffer_declaration_numpy.sifr",
            },
            {
                "detach before release",
                "exact-once PyBuffer_Release",
                "exporter retention",
                "zero live/leaked resources",
            },
        ),
        "automatic-drop": (
            "runtime-native-binary",
            "C-level automatic release counter plus six retained compiled exporters resizable after aggregate drop",
            {
                "crates/sifr_runtime/src/python/buffer_ops/release_evidence_tests.rs::instrumented_exporter_automatic_resource_drop_releases_exactly_once",
                "verification/areas/python_interop/fixtures/numpy_buffer/buffer_affine_aggregate_codegen.sifr",
            },
            {
                "normal return",
                "aggregate field drop",
                "Option/list/tuple/union drop",
                "recursive aggregate drop",
                "zero live/leaked resources",
            },
        ),
        "failure-rollback": (
            "runtime",
            "instrumented validation, admission-conflict, and store-failure release counters",
            {
                "crates/sifr_runtime/src/python/buffer_ops/release_evidence_tests.rs::instrumented_exporter_validation_failure_rolls_back_exactly_once",
                "crates/sifr_runtime/src/python/buffer_ops/release_evidence_tests.rs::instrumented_exporter_admission_conflict_releases_rejected_view_exactly_once",
                "crates/sifr_runtime/src/python/buffer_ops/release_evidence_tests.rs::instrumented_exporter_store_failure_rolls_back_exactly_once",
            },
            {
                "validation failure",
                "admission conflict",
                "store failure rollback",
                "exact-once rejected-view release",
                "lock-free Python release",
            },
        ),
    },
}
