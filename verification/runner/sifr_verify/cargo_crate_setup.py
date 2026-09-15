"""Prepare the exact selected crate test graphs before timed execution."""

from .profiles import crate_test_mode, crate_test_suites_for_mode


def prepare_crate_test_binaries(profile, env, command_runner) -> None:
    mode = crate_test_mode(profile)
    if mode is None:
        return
    for suite in crate_test_suites_for_mode(profile, mode):
        if suite["status"] == "red-blocker" and not suite["executed_in_merge"]:
            continue
        original = suite["command"]
        if not isinstance(original, list) or not original or original[0] != "test":
            raise ValueError("crate preparation requires a canonical cargo test command")
        # Arguments after -- belong to the test executable, not Cargo's graph.
        arguments = original[1:]
        if "--" in arguments:
            arguments = arguments[:arguments.index("--")]
        command = ["cargo", "test", "--locked", "--offline", "--no-run", *arguments]
        print(f"[sifr-profile-setup] crate-test-build={' '.join(command)}", flush=True)
        command_runner(command, env=env)
