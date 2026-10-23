"""Explicit preparation ownership within the native libtest inventory."""
import re
from .errors import VerificationError

ADVANCED_DATA_PREPARATION = (
    "tests::package_project_build_check::rust_interop_build_tests::"
    "advanced_data_support::prepare_advanced_data_native_graph"
)

def preparation_cases(configuration):
    return (ADVANCED_DATA_PREPARATION,) if "sifr_driver_generated_builds" in configuration.ids else ()

def require_one_executed_case(outcome, name):
    if outcome.cause != "exit" or outcome.returncode or outcome.truncated or not re.search(
        rb"^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; \d+ filtered out;",
        outcome.stdout, re.MULTILINE,
    ):
        raise VerificationError(f"native case did not attest exactly one execution: {name}")
