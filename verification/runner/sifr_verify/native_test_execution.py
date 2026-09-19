"""Execute the existing isolated native libtest selection with per-case safety bounds.

The selected inventory and Cargo feature/target flags remain authoritative.
Safety deadlines bound each child; they are not an aggregate performance budget.
"""
import re
from .errors import VerificationError
from .profile_commands import CommandFailed, run_command

NATIVE_SUITES = frozenset({"sifr_cli_generated_builds", "sifr_driver_generated_builds"})

def selected_native_cases(outcome):
    if outcome.cause != "exit" or outcome.returncode or outcome.truncated:
        raise VerificationError("native test inventory did not complete without truncation")
    try:
        lines = outcome.stdout.decode("utf-8", errors="strict").strip().splitlines()
    except UnicodeDecodeError as error:
        raise VerificationError("native test inventory is not UTF-8") from error
    if not lines:
        raise VerificationError("empty native test inventory")
    summary = re.fullmatch(r"(\d+) tests?, 0 benchmarks?", lines[-1])
    names = []
    for line in lines[:-1]:
        if not line:
            continue
        match = re.fullmatch(r"([A-Za-z_][A-Za-z_0-9:]*): test", line)
        if match is None:
            raise VerificationError(f"invalid native test inventory line: {line!r}")
        names.append(match[1])
    if not summary or int(summary[1]) != len(names) or not names or len(set(names)) != len(names):
        raise VerificationError("native test inventory is empty, duplicate, or incomplete")
    return tuple(names)

def run_native_configuration(configuration, *, env, no_fail_fast):
    command = configuration.execution()
    if configuration.ids[0] not in NATIVE_SUITES or len(configuration.ids) != 1:
        raise VerificationError("native case scheduling requires one isolated suite")
    separator = command.index("--")
    if command[separator + 1:] != ["--ignored", "--test-threads=1"]:
        raise VerificationError("native case scheduling selection changed; review its flags")
    names = selected_native_cases(run_command([*command, "--list"], env=env))
    first_failure = None
    for name in names:
        status = "pass"
        try:
            outcome = run_command([*command, "--exact", name], env=env)
            if outcome.truncated or not re.search(
                rb"^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; \d+ filtered out;",
                outcome.stdout, re.MULTILINE,
            ):
                raise VerificationError(f"native case did not attest exactly one executed assertion: {name}")
        except (CommandFailed, VerificationError) as error:
            status = "fail"
            first_failure = first_failure or error
            if not no_fail_fast:
                raise
        finally:
            print(f"[sifr-native-case] suite={configuration.ids[0]} case={name} status={status}")
    if first_failure:
        raise first_failure
