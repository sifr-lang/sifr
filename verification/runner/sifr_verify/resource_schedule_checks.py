"""Capacity, ownership and complete staged-selection regressions."""
from __future__ import annotations

import json
import shutil
import sys
import subprocess
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from .cloud_schedule import Schedule, clamp_workers, load_schedule, run_staged_cloud
from .cloud_failure import classify_failure
from .sysroot_preparation import producer, protected_compilers
from .profile_commands import CommandFailed
from .cargo_setup import acquire_cargo_dependencies
from .graph_retirement import GRAPH_PATHS, GraphLease
from .sql_resource_checks import SqlResourceTests
from .sql_worker_checks import SqlWorkerChecks
from .early_sql_checks import EarlySqlChecks
from .sql_partition_checks import PartitionChecks
from .sql_cli_preparation_checks import SqlCliPreparationChecks
from .early_sql import EarlySqlOutcome
from .profile_runner import ProfileRunner
from .resource_admission import ResourceError, Resources, admit, discover, own_cgroup, worker_limit, memory_backed_storage


@unittest.skipUnless(sys.platform.startswith("linux"), "cloud resource contract is Linux cgroup v2")
class ResourceTests(unittest.TestCase):
    def test_every_e2e_worker_argument_is_clamped_without_changing_inventory(self):
        runner = ProfileRunner("cloud", ["--sifr-jobs=12", "--run-jobs", "1"])
        canonical = runner.profile["e2e"].copy()
        resources = Resources(5, 1.5, 100, 100, 100, {}, [], {})
        limits = clamp_workers(runner, resources)
        self.assertEqual(limits, {"sifr_jobs": 1, "rust_jobs": 1, "run_jobs": 1, "cargo_build_jobs": 1})
        self.assertEqual(runner.profile["e2e"], canonical)
        with patch("sifr_verify.profile_runner.run_command") as command:
            runner.run_e2e_pass_suite()
        argv = command.call_args.args[0]
        for flag in ("--sifr-jobs", "--rust-jobs", "--run-jobs", "--cargo-build-jobs"):
            self.assertEqual(argv.count(flag), 1)
            self.assertEqual(argv[argv.index(flag) + 1], "1")
        self.assertEqual(clamp_workers(runner, resources), limits)

    def test_oom_requires_kill_and_counter_evidence_and_enospc_is_preserved(self):
        error = CommandFailed(101, "exit")
        error.outcome = SimpleNamespace(stderr=b"rustc (signal: 9, SIGKILL)")
        before = {"/group/memory.events": "oom_kill 0"}
        after = {"/group/memory.events": "oom_kill 1"}
        self.assertEqual(classify_failure(error, before, after), "oom")
        self.assertEqual(classify_failure(error, before, before), "assertion")
        error.outcome = SimpleNamespace(stderr=b"No space left on device")
        self.assertEqual(classify_failure(error, before, before), "enospc")
        for classification in ("enospc", "admission", "unavailable"):
            error.outcome = SimpleNamespace(stderr=f"sysroot-preparation: infrastructure={classification} detail".encode())
            self.assertEqual(classify_failure(error, before, before), classification)
        self.assertEqual(classify_failure(KeyboardInterrupt(), before, before), "cancelled")

    def test_nested_cpu_memory_limits_affinity_and_tmpfs_share_one_budget(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            child = root / "child"
            child.mkdir()
            meminfo = root / "meminfo"
            meminfo.write_text("MemTotal: 32768 kB\nMemAvailable: 24000 kB\n")
            for path, cpu, memory, used in [(root, "200000 100000", 16000, 3000),
                                             (child, "350000 100000", 20000, 1000)]:
                (path / "cpu.max").write_text(cpu)
                (path / "memory.max").write_text(str(memory))
                (path / "memory.current").write_text(str(used))
            resources = discover(disk_path=root, cgroup_root=root, cgroup_path=child,
                                 meminfo=meminfo, affinity=5, tmpfs_paths=())
            self.assertEqual((resources.effective_cpus, resources.memory_limit_bytes,
                              resources.memory_available_bytes), (2, 16000, 13000))
            self.assertEqual(worker_limit(resources, 8), 2)
            requirements = dict(disk_growth_bytes=1, retained_copy_bytes=1, disk_reserve_bytes=1,
                                memory_peak_bytes=9000, tmpfs_growth_bytes=5000, memory_reserve_bytes=0)
            with self.assertRaisesRegex(ResourceError, "shared resident/tmpfs"):
                admit(resources, requirements)
            (root / "memory.stat").write_text("inactive_file 2000\nfile_dirty 300\nfile_writeback 200\nunevictable 100\n")
            reclaimed = discover(disk_path=root, cgroup_root=root, cgroup_path=child,
                                 meminfo=meminfo, affinity=5, tmpfs_paths=())
            self.assertEqual(reclaimed.memory_available_bytes, 14400)
            self.assertEqual(reclaimed.memory_limit_bytes, 16000)
            (root / "memory.stat").write_text(
                "inactive_file 2000\nactive_file 1000\nfile_dirty 300\nfile_writeback 200\nunevictable 100\n"
                "shmem 9000\nactive_anon 9000\n")
            active = discover(disk_path=root, cgroup_root=root, cgroup_path=child,
                              meminfo=meminfo, affinity=5, tmpfs_paths=())
            self.assertEqual(active.memory_available_bytes, 15400)
            self.assertEqual(active.memory_limit_bytes, 16000)
            (root / "memory.stat").write_text("inactive_file 2000\nactive_file 1000\nfile_dirty 3000\n")
            dirty = discover(disk_path=root, cgroup_root=root, cgroup_path=child,
                             meminfo=meminfo, affinity=5, tmpfs_paths=())
            self.assertEqual(dirty.memory_available_bytes, 13000)

    def test_unknown_cgroup_and_traversal_are_not_host_capacity(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            membership = root / "membership"
            membership.write_text("0::/nested/job\n")
            self.assertEqual(own_cgroup(root, membership), root / "nested/job")
            membership.write_text("0::/../other\n")
            with self.assertRaises(ValueError):
                own_cgroup(root, membership)
            with self.assertRaises(ResourceError):
                discover(disk_path=root, cgroup_root=root, cgroup_path=root)

    def test_memory_backed_build_storage_is_charged_with_resident_and_other_tmpfs(self):
        resources = Resources(4, 4, 100, 20, 100, {}, [], {}, True)
        requirements = dict(disk_growth_bytes=8, retained_copy_bytes=5, disk_reserve_bytes=8,
                            memory_peak_bytes=5, tmpfs_growth_bytes=3, memory_reserve_bytes=1)
        with self.assertRaisesRegex(ResourceError, "shared resident/tmpfs"):
            admit(resources, requirements)
        ordinary = Resources(4, 4, 100, 20, 100, {}, [], {})
        self.assertEqual(admit(ordinary, requirements)["memory_admitted_bytes"], 9)
        requirements["disk_growth_bytes"] = 6
        accepted = admit(resources, requirements)
        self.assertEqual(accepted["memory_admitted_bytes"], 20)
        self.assertEqual(accepted["memory_backed_storage_growth_bytes"], 11)
        self.assertEqual(requirements["tmpfs_growth_bytes"], 3)

    def test_nested_mount_resolution_and_escaped_paths(self):
        with tempfile.TemporaryDirectory(prefix="mount control ") as directory:
            root = Path(directory)
            nested = root/"disk";nested.mkdir()
            mounts = root/"mountinfo"
            escaped = str(root).replace(" ", r"\040")
            mounts.write_text("1 0 0:1 / / rw - overlay overlay rw\n"
                              +f"2 1 0:2 / {escaped} rw - tmpfs tmpfs rw\n"
                              +f"3 2 0:3 / {escaped}/disk rw - ext4 /dev/test rw\n")
            self.assertTrue(memory_backed_storage(root, mounts))
            self.assertFalse(memory_backed_storage(nested, mounts))
            mounts.write_text("")
            with self.assertRaises(ValueError): memory_backed_storage(root, mounts)

    def test_retained_disk_copies_and_reserve_are_admitted_before_work(self):
        resources = Resources(4, 4, 100, 100, 20, {}, [], {})
        requirements = dict(disk_growth_bytes=8, retained_copy_bytes=5, disk_reserve_bytes=8,
                            memory_peak_bytes=1, tmpfs_growth_bytes=0, memory_reserve_bytes=1)
        with self.assertRaises(ResourceError) as raised:
            admit(resources, requirements)
        self.assertEqual(raised.exception.classification, "enospc")


@unittest.skipUnless(sys.platform.startswith("linux"), "cloud graph leases require Linux")
class RetirementTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.lease = GraphLease(self.root, GRAPH_PATHS[0], "session").acquire({"boundary", "installed"})
        self.addCleanup(self.lease.close)
        self.binary = self.lease.path / "debug/sifr"
        self.binary.parent.mkdir()
        self.binary.write_bytes(b"compiler")

    def retire(self, **kwargs):
        return self.lease.retire(retained=self.root / "retained", protected=[self.binary],
            command_runner=lambda command, env: shutil.rmtree(self.lease.path), env={}, **kwargs)

    def complete(self):
        for identifier in self.lease.consumers:
            self.lease.passed_consumer(identifier)

    def test_failed_or_incomplete_consumers_and_live_builds_prevent_cleanup(self):
        self.lease.passed_consumer("boundary")
        with self.assertRaises(ResourceError):
            self.retire()
        self.assertTrue(self.binary.exists())

    def test_leased_target_is_accepted_by_the_actual_pinned_cargo_clean(self):
        result = subprocess.run(["cargo", "clean", "--target-dir", str(self.lease.path), "--dry-run"],
                                capture_output=True, text=True, timeout=30)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(self.binary.exists())
        self.complete()
        with patch("sifr_verify.graph_retirement.active_builds", return_value=[123]):
            with self.assertRaises(ResourceError):
                self.retire()
        self.assertTrue(self.binary.exists())

    def test_lease_contention_other_owner_and_marker_drift_are_rejected(self):
        contender = GraphLease(self.root, GRAPH_PATHS[0], "other")
        self.addCleanup(contender.close)
        with self.assertRaises(BlockingIOError):
            contender.acquire({"boundary"})
        self.complete()
        marker = json.loads(self.lease.marker.read_text())
        marker["owner"] = "other"
        self.lease.marker.write_text(json.dumps(marker))
        with self.assertRaises(ResourceError):
            self.retire()
        self.assertTrue(self.binary.exists())

    def test_symlinks_cannot_redirect_clean_or_retention(self):
        self.complete()
        (self.lease.path / "outside").symlink_to(self.root, target_is_directory=True)
        with self.assertRaises(ResourceError):
            self.retire()
        (self.lease.path / "outside").unlink()
        (self.root / "retained").symlink_to(self.lease.path, target_is_directory=True)
        with self.assertRaises(ResourceError):
            self.retire()

    def test_unknown_legacy_graph_is_never_reclaimed(self):
        self.lease.close()
        self.lease.marker.unlink()
        (self.lease.path / "CACHEDIR.TAG").unlink()
        lease = GraphLease(self.root, GRAPH_PATHS[0], "session").acquire({"boundary"})
        self.addCleanup(lease.close)
        lease.passed_consumer("boundary")
        result = lease.retire(retained=self.root / "retained", protected=[self.binary],
                              command_runner=lambda *args, **kwargs: self.fail("legacy cleanup"), env={})
        self.assertFalse(result["retired"])
        self.assertTrue(self.binary.exists())
        self.assertFalse((lease.path / "CACHEDIR.TAG").exists())

    def test_only_explicit_prior_ownership_can_recover_a_cancelled_graph(self):
        self.lease.close()
        wrong = GraphLease(self.root, GRAPH_PATHS[0], "new-session").acquire({"boundary"})
        self.assertFalse(wrong.eligible)
        wrong.close()
        recovered = GraphLease(self.root, GRAPH_PATHS[0], "session").acquire({"boundary"})
        self.addCleanup(recovered.close)
        self.assertTrue(recovered.eligible)
        self.assertEqual(recovered.passed, set())

    def test_retention_uses_both_real_producers_including_the_host_triple(self):
        source, package = producer("source_build"), producer("package_build")
        host = "x86_64-unknown-linux-gnu"
        with patch("sifr_verify.sysroot_preparation.producer", side_effect=[source, package]), \
             patch.object(package, "host_target", return_value=host):
            compilers = protected_compilers(self.root, {})
        command, environment = package.package_build_configuration(self.root, {}, host, self.root / "artifacts")
        self.assertEqual(compilers["cargo-target"], Path(environment["CARGO_TARGET_DIR"]) /
                         command[command.index("--target") + 1] / "release/sifr")
        self.assertEqual(compilers["source-cargo-target"], self.binary)
        _, source_env, _ = source.source_build_configuration(self.root, {"CARGO_INCREMENTAL": "1"})
        self.assertEqual(source_env["CARGO_INCREMENTAL"], "0")
        lease = GraphLease(self.root, GRAPH_PATHS[1], "session").acquire({"installed"})
        self.addCleanup(lease.close)
        binary = compilers["cargo-target"]
        binary.parent.mkdir(parents=True)
        binary.write_bytes(b"host-qualified-compiler")
        lease.passed_consumer("installed")
        with patch("sifr_verify.graph_retirement.active_builds", return_value=[]):
            result = lease.retire(retained=self.root / "retained", protected=[binary], env={},
                command_runner=lambda command, env: shutil.rmtree(Path(command[-1])))
        self.assertTrue(result["retired"])
        self.assertEqual(Path(result["copies"][0]["retained"]["path"]).read_bytes(), b"host-qualified-compiler")

    def test_net_recovery_includes_copies_and_immutable_compiler_survives(self):
        self.complete()
        with patch("sifr_verify.graph_retirement.active_builds", return_value=[]), \
             patch("sifr_verify.graph_retirement.shutil.disk_usage", side_effect=[
                 SimpleNamespace(free=100), SimpleNamespace(free=140)]):
            result = self.retire()
        self.assertEqual(result["reclaimed_bytes"], 40)
        kept = Path(result["copies"][0]["retained"]["path"])
        self.assertEqual(kept.read_bytes(), b"compiler")
        self.assertEqual(kept.stat().st_mode & 0o222, 0)
        self.assertFalse(self.lease.path.exists())


@unittest.skipUnless(sys.platform.startswith("linux"), "cloud scheduler requires Linux")
class ScheduleTests(unittest.TestCase):
    def test_preparation_cannot_extend_an_inherited_deadline_and_assertions_keep_per_command_bounds(self):
        schedule = object.__new__(Schedule)
        schedule.policy = load_schedule()
        observed = []
        runner = SimpleNamespace(env={"SIFR_VERIFY_SAFETY_DEADLINE_SECONDS": "17"})
        def execute(name, callback):
            observed.append(runner.env.copy())
            return 0
        runner.execute_step = execute
        schedule.runner = runner
        schedule.step("prepare", lambda: None, allocation="sysroot-source", preparation=True)
        schedule.step("assert", lambda: None, allocation="sysroot-assertions")
        self.assertEqual(observed[0]["SIFR_VERIFY_STEP_SAFETY_DEADLINE_SECONDS"], "17.0")
        self.assertNotIn("SIFR_VERIFY_STEP_SAFETY_DEADLINE_SECONDS", observed[1])
        self.assertEqual(observed[1]["SIFR_VERIFY_SAFETY_DEADLINE_SECONDS"], "17.0")
        self.assertEqual(runner.env, {"SIFR_VERIFY_SAFETY_DEADLINE_SECONDS": "17"})

    def test_acquisition_cannot_compile_a_producer_before_graph_retirement(self):
        commands = []
        profile = ProfileRunner("cloud", []).profile
        acquire_cargo_dependencies(profile, {}, lambda command, env: commands.append(command))
        self.assertTrue(commands)
        self.assertTrue(all(command[:2] == ["cargo", "fetch"] for command in commands))

    def test_all_canonical_assertions_survive_earlier_preparation_output_retirement(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result = root / "target/verification/areas/sysroot-release-cloud-results.json"
            result.parent.mkdir(parents=True)
            result.write_text('{"passed":true}')
            runner = ProfileRunner("cloud", [])
            events = []
            early = {"hir-maintainability", "file-size", "source-crate-dependency-direction",
                     "submodule-ownership", "stdlib-manifest-schema"}
            class FakeSchedule:
                owner = "test"
                graph_owner = "test"
                journal = root / "journal"
                def __init__(self, runner): pass
                def record(self, *args): pass
                def prepare_command(self, command, *, env):
                    preparations.append(command)
                def step(self, name, callback, **kwargs):
                    events.append(name)
                    callback()
                    return 0
            preparations = []
            with patch("sifr_verify.cloud_schedule.run_early_sql", return_value=EarlySqlOutcome()), \
                 patch("sifr_verify.cloud_schedule.REPO_ROOT", root), \
                 patch("sifr_verify.cloud_schedule.Schedule", FakeSchedule), \
                 patch("sifr_verify.cloud_schedule.run_command", side_effect=lambda argv, env: preparations.append(argv)), \
                 patch("sifr_verify.cloud_schedule.acquire_cargo_dependencies"), \
                 patch("sifr_verify.cloud_schedule.prepare_remaining_graphs") as remaining, \
                 patch.object(runner, "run_guardrail") as guards, \
                 patch.object(runner, "run_area") as areas, \
                 patch.object(runner, "run_toolchain_step") as tools:
                self.assertEqual(run_staged_cloud(runner, early), 0)
            self.assertEqual(guards.call_count, len(runner.profile["guardrail_steps"]) - len(early))
            self.assertEqual(areas.call_args_list[0].args[0], "sysroot_release")
            self.assertEqual(areas.call_count, len(runner.profile["selected_areas"]))
            self.assertEqual(tools.call_count, len(runner.profile["toolchain_steps"]))
            self.assertEqual(len(events), len(set(events)))
            self.assertLess(events.index("preparation_sysroot_source"), events.index("preparation_sysroot_package"))
            self.assertLess(events.index("preparation_sysroot_package"), events.index("preparation_sysroot_metadata"))
            self.assertLess(events.index("preparation_sysroot_metadata"), events.index("area_sysroot_release"))
            self.assertEqual([argv[-2:] for argv in preparations[:2]], [["prepare", "source"], ["prepare", "package"]])
            self.assertEqual(preparations[2][-1], "--metadata-only")
            self.assertFalse(remaining.call_args.kwargs["include_sysroot"])


def policy_checks():
    suite = unittest.TestSuite(unittest.defaultTestLoader.loadTestsFromTestCase(case)
                               for case in (ResourceTests, RetirementTests, ScheduleTests, SqlResourceTests, EarlySqlChecks, PartitionChecks, SqlCliPreparationChecks))
    result = unittest.TestResult()
    suite.run(result)
    if not result.wasSuccessful():
        raise AssertionError(result.errors + result.failures)


if __name__ == "__main__":
    unittest.main()
