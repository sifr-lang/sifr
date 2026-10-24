"""Execute the existing isolated native libtest selection with per-case safety bounds.

The selected inventory and Cargo feature/target flags remain authoritative.
Safety deadlines bound each child; they are not an aggregate performance budget.
"""
import re
from .errors import VerificationError
from .native_test_roles import preparation_cases, require_one_executed_case
from .profile_commands import CommandFailed, run_command
from .native_test_allocation import native_environment, readmit, reserve_monitor
from .native_command_batch import run_native_batch

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

def run_native_configuration(configuration, *, env, no_fail_fast, preparation_complete):
    command = configuration.execution()
    if configuration.ids[0] not in NATIVE_SUITES or len(configuration.ids) != 1:
        raise VerificationError("native case scheduling requires one isolated suite")
    separator = command.index("--")
    if command[separator + 1:] != ["--ignored", "--test-threads=1"]:
        raise VerificationError("native case scheduling selection changed; review its flags")
    allocation, env = native_environment(env)
    workers = readmit(allocation)
    names = selected_native_cases(run_command([*command, "--list"], env=env))
    helpers = preparation_cases(configuration)
    if not set(helpers) <= set(names):
        raise VerificationError("declared native preparation is missing from inventory")
    if helpers and not preparation_complete:
        raise VerificationError("native assertions require successful current-run crate preparation")
    names = tuple(name for name in names if name not in helpers)
    if not names:
        raise VerificationError("native selection contains no assertion cases")
    for helper in helpers:
        print(f"[sifr-native-selection] suite={configuration.ids[0]} case={helper} "
              "role=preparation owner=crate_tests")
    results = {}
    def completed(index, outcome):
        if index in results or index < 0 or index >= len(names):
            raise VerificationError("duplicate or invalid native outcome index")
        error = None
        try:
            if outcome.cause != "exit" or outcome.returncode:
                raise CommandFailed(outcome.returncode or 2, outcome.cause)
            require_one_executed_case(outcome, names[index])
        except (CommandFailed, VerificationError) as failure:
            error = failure
        results[index] = error
        if error is not None and (not no_fail_fast or outcome.cause != "exit"
                                  or outcome.returncode < 0 or outcome.truncated
                                  or isinstance(error, VerificationError)):
            raise error
    try:
        run_native_batch([[*command, "--exact", name] for name in names], env=env,
                         workers=workers, completed=completed,
                         monitor=reserve_monitor(allocation))
        if len(results) != len(names):
            raise VerificationError("native execution returned incomplete selection")
    finally:
        for index, name in enumerate(names):
            status = "incomplete" if index not in results else "fail" if results[index] else "pass"
            print(f"[sifr-native-case] suite={configuration.ids[0]} case={name} status={status}", flush=True)
    for index in range(len(names)):
        if results[index] is not None:
            raise results[index]
