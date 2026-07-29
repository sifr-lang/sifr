"""Required Rust interop scenario packages and their structural tokens."""

from _scenario_advanced_data import ADVANCED_DATA_SCENARIO_TOKENS
from _scenario_callback_subscriptions import CALLBACK_SUBSCRIPTION_SCENARIO_TOKENS
from _scenario_cargo_locked import CARGO_LOCKED_SCENARIO_TOKENS
from _scenario_native_build import NATIVE_BUILD_SCENARIO_TOKENS
from _scenario_opaque_resources import OPAQUE_RESOURCE_SCENARIO_TOKENS
from _scenario_zero_copy import ZERO_COPY_SCENARIO_TOKENS

REQUIRED_SCENARIO_EXAMPLES = {
    "advanced_data_runtime_matrix": {
        "advanced_data_runtime": {"tokens": ADVANCED_DATA_SCENARIO_TOKENS},
    },
    "async_runtime_reqwest": {
        "reqwest_loopback_runtime": {
            "tokens": (
                "reqwest::Client",
                ".no_proxy()",
                "127.0.0.1",
                "task.timeout",
                "runtime_reused",
                "handle.id()",
                "ring_core_0_17_14_",
            ),
        },
    },
    "bridge_type_matrix": {
        "bridge_type_roundtrip": {
            "tokens": (
                "serde_json_roundtrip",
                "bytes_roundtrip",
                "indexmap_roundtrip",
                "nested_indexmap_roundtrip",
                "indexmap_list_roundtrip",
                "thiserror",
            ),
        },
    },
    "bridge_version_mismatch": {
        "bridge_version_package": {
            "tokens": ("bridge-version = 1", "version_bridge"),
        },
    },
    "callbacks_call_scoped": {
        "call_scoped_callback_runtime": {
            "tokens": (
                "CallScopedCallbackBridge",
                "bridge.callbacks.visit",
                "Rust bridge panicked",
            ),
        },
    },
    "callback_subscription_ecosystem": {
        "subscription_lifecycle_runtime": {
            "tokens": CALLBACK_SUBSCRIPTION_SCENARIO_TOKENS,
        },
    },
    "cargo_locked_offline": {
        "locked_offline_cache": {
            "tokens": CARGO_LOCKED_SCENARIO_TOKENS,
        },
    },
    "ecosystem_backend_certification": {
        "backend_feature_package": {
            "tokens": ("runtime-tokio-rustls", "postgres", "macros", "tower-http"),
        },
    },
    "ecosystem_cli_certification": {
        "cli_feature_package": {
            "tokens": ("env-filter", "tracing-subscriber", "clap"),
        },
    },
    "local_bridge_blake3": {
        "local_blake3_bridge": {
            "tokens": ("bridge.blake3.hash_bytes", "src/bridges", "blake3"),
        },
    },
    "native_build_script": {
        "native_trust_package": {"tokens": NATIVE_BUILD_SCENARIO_TOKENS},
    },
    "opaque_resource_matrix": {
        "resource_lifecycle_runtime": {
            "tokens": OPAQUE_RESOURCE_SCENARIO_TOKENS,
        },
    },
    "panic_abort_profile": {
        "abort_profile_package": {
            "tokens": ("rust-panic-abort", 'panic = "abort"', "legacy_backend"),
        },
    },
    "panic_boundary_wrapper_emission": {
        "panic_wrapper_runtime": {
            "tokens": (
                "RustPanicErrorBridge",
                "mapper_panics",
                "--locked",
                "--offline",
                "--frozen",
            ),
        },
    },
    "proc_macro_trust": {
        "proc_macro_trust_package": {
            "tokens": (
                "rust-proc-macros",
                "rust-build-scripts",
                "serde_derive_upstream",
                "prost_build_upstream",
                "compile_fds",
                "SifrGenerated",
            ),
        },
    },
    "same_workspace_crate": {
        "workspace_hash_crate": {
            "tokens": (
                "workspace_hash",
                'path = "rust/workspace_hash"',
                "members = [",
            ),
        },
    },
    "shared_bridge_crate": {
        "shared_hash_bridge": {
            "tokens": (
                "sifr_shared_hash_bridge",
                "digest_hex",
                "crate::__sifr_bridge",
            ),
        },
    },
    "zero_copy_runtime_matrix": {
        "crate_backed_view_runtime": {"tokens": ZERO_COPY_SCENARIO_TOKENS},
    },
}
