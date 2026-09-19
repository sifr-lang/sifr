"""Prepare the exact selected crate test graphs before timed execution."""

from .profiles import crate_test_mode
from .compiler_configuration_plan import configuration_plan


def prepare_crate_test_binaries(profile, env, command_runner) -> None:
    mode = crate_test_mode(profile)
    if mode is None:
        return
    prepared = set()
    built = set()
    for configuration in configuration_plan(profile, mode):
        command = configuration.preparation()
        arguments = command[5:]
        if tuple(command) not in built:
            print(f"[sifr-profile-setup] crate-test-build={' '.join(command)}", flush=True)
            command_runner(command, env=env)
            built.add(tuple(command))

        # Run the already linked driver setup in this exact Cargo test graph.
        # No CLI binary, recursive Cargo producer, or cross-configuration override.
        if "sifr_driver" in arguments or "--workspace" in arguments:
            graph = tuple(arguments)
            if graph not in prepared:
                command_runner(["cargo", "test", "--locked", "--offline", *arguments,
                                "metadata_producer::tests::dx6_prepare_test_metadata",
                                "--", "--exact", "--nocapture"], env=env)
                prepared.add(graph)

        if "sifr_driver_generated_builds" in configuration.ids:
            # The copied fixture authority has one leased worktree-owned path,
            # so preparation and runtime assertions use the same native family.
            # Preparation costs are charged separately; every runtime assertion
            # still executes under its existing deadline.
            test = ("tests::package_project_build_check::rust_interop_build_tests::"
                    "advanced_data_support::prepare_advanced_data_native_graph")
            command_runner(["cargo", "test", "--locked", "--offline", *arguments,
                            test, "--", "--exact", "--ignored", "--nocapture"], env=env)
