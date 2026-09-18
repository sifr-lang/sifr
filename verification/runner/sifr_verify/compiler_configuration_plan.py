"""One immutable Cargo selection for preparation and execution.

Only additive compiler integration packages are grouped. Runtime/default-feature,
SQL/native, feature-isolation and explicit target selections retain their boundary.
The plan carries every original assertion/filter, never an all-features application.
"""
from dataclasses import dataclass
from .profiles import crate_test_suites_for_mode

ADDITIVE_COMPILER_PACKAGES = frozenset({
    "sifr_identity", "sifr_diagnostics", "sifr_lowering", "sifr_syntax",
    "sifr_frontend", "sifr_analysis", "sifr_lsp", "sifr_package", "sifr_ipc",
    "sifr_stdlib_imports", "sifr_stdlib_manifest", "sifr_type_system",
    "sifr_format", "sifr_lint", "sifr_source", "sifr_sysroot", "sifr_ir",
    "sifr_codegen", "sifr_driver", "sifr",
})

@dataclass(frozen=True)
class Configuration:
    ids: tuple[str, ...]
    command: tuple[str, ...]
    originals: tuple[tuple[str, tuple[str, ...]], ...]
    classification: str

    def preparation(self):
        arguments = self.command[1:]
        if "--" in arguments:
            arguments = arguments[:arguments.index("--")]
        return ["cargo", "test", "--locked", "--offline", "--no-run", *arguments]

    def execution(self):
        return ["cargo", self.command[0], "--locked", "--offline", *self.command[1:]]

def configuration_plan(profile, mode):
    groups = {}
    for suite in crate_test_suites_for_mode(profile, mode):
        if suite["status"] == "red-blocker" and not suite["executed_in_merge"]:
            continue
        command = tuple(suite["command"])
        if not command or command[0] != "test":
            raise ValueError("compiler configuration requires cargo test")
        package = command[2] if len(command) >= 3 and command[1] == "-p" else None
        rest = command[3:]
        # Explicit --lib/--bin/--target/feature/native/ignored cases stay isolated.
        additive = (package in ADDITIVE_COMPILER_PACKAGES and
                    rest in ((), ("--", "--skip", "test_e2e_pass")))
        key = ("integration", rest) if additive else ("isolated", suite.get("id", repr(command)))
        groups.setdefault(key, []).append(suite)
    result = []
    for (classification, _), suites in groups.items():
        if classification == "integration":
            rest = tuple(suites[0]["command"][3:])
            command = ("test", *(arg for s in suites for arg in ("-p", s["command"][2])), *rest)
        else:
            command = tuple(suites[0]["command"])
        originals = tuple((s.get("id", repr(s["command"])), tuple(s["command"])) for s in suites)
        result.append(Configuration(tuple(x[0] for x in originals), command, originals, classification))
    return tuple(result)
