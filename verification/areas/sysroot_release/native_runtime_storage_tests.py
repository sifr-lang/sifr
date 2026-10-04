"""Real small paths and environments; physical OS topology is fixture-controlled."""
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT/'verification/runner'))
import native_runtime_storage as policy
from sifr_verify.process_execution import execute

TOPOLOGY = {'storage_device': '/dev/disk3s1', 'filesystem': 'apfs', 'volume_bus_protocol': '',
            'apfs_backing_stores': [{'DeviceNode': '/dev/disk0s2', 'BusProtocol': 'Apple Fabric'}]}


class StorageTests(unittest.TestCase):
    def test_real_env_paths_and_parent_temp_restore(self):
        with tempfile.TemporaryDirectory() as temporary, patch.object(policy, 'storage', return_value=TOPOLOGY):
            root = Path(temporary).resolve()/'owned'
            env, proof = policy.prepare(root, {})
            cached = tempfile.tempdir
            with policy.parent_temporary(proof):
                with tempfile.TemporaryFile() as stream:
                    stream.write(b'unit')
                self.assertEqual(tempfile.gettempdir(), str(root/'temporary'))
                result = execute([sys.executable, '-B', '-c',
                    "import os,tempfile;print(tempfile.gettempdir());print(os.environ['CARGO_BUILD_JOBS'])"],
                    cwd=root, env=env | {'SIFR_VERIFY_DISK_FLOOR_BYTES': '1'})
                self.assertIn(str(root/'temporary').encode(), result.stdout)
                self.assertTrue(result.stdout.endswith(b'1\n'))
            self.assertEqual(tempfile.tempdir, cached)
            policy.check(proof, root, env)
            (root/'sifr-cache/escape').symlink_to(root/'temporary', target_is_directory=True)
            with self.assertRaises(ValueError): policy.check(proof, root, env)

    def test_inherited_configs_and_overrides_reject(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            for name in ('RUSTFLAGS', 'CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER', 'SIFR_SYSROOT', 'CC_arm64', 'PYTHONPATH'):
                with self.subTest(name=name), self.assertRaises(ValueError):
                    policy.configuration(root, {name: 'uncontrolled'})
            policy.configuration(root, {'CARGO_PROFILE_DEV_DEBUG': '0', 'CARGO_BUILD_JOBS': '1'})
            (root/'.cargo').mkdir(); (root/'.cargo/config.toml').write_text('[env]\nTMPDIR="/foreign"\n')
            with self.assertRaises(ValueError): policy.configuration(root, {})

    def test_exact_published_vendor_config_is_allowed(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            folder = root/'package/.cargo'; folder.mkdir(parents=True)
            (folder/'config.toml').write_text('[source.crates-io]\nreplace-with="sifr-vendor"\n[source.sifr-vendor]\ndirectory="vendor"\n')
            policy.configuration(root, {})
            (folder/'config.toml').write_text('[source.crates-io]\nreplace-with="foreign"\n')
            with self.assertRaises(ValueError): policy.configuration(root, {})

    def test_reserves_and_full_physical_temp_allocation(self):
        value = policy.requirements({'size': 123})
        self.assertEqual(value['memory_peak_bytes'], 768*1024**2)
        self.assertEqual(value['memory_reserve_bytes'], 2*1024**3)
        self.assertEqual(value['disk_reserve_bytes'], 8*1024**3)
        self.assertEqual(value['tmpfs_growth_bytes'], 0)
        self.assertGreater(value['disk_growth_bytes'], 10*1024**3)

    def test_stricter_owned_disk_floor_is_preserved_and_foreign_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            inherited = {'SIFR_VERIFY_DISK_FLOOR_PATH': str(root),
                         'SIFR_VERIFY_DISK_FLOOR_BYTES': str(9*1024**3)}
            self.assertEqual(policy.environment(root, inherited)['SIFR_VERIFY_DISK_FLOOR_BYTES'], str(9*1024**3))
            inherited['SIFR_VERIFY_DISK_FLOOR_PATH'] = str(root.parent)
            with self.assertRaisesRegex(ValueError, 'same owned root'): policy.environment(root, inherited)
            inherited.pop('SIFR_VERIFY_DISK_FLOOR_BYTES')
            with self.assertRaises(ValueError): policy.environment(root, inherited)


if __name__ == '__main__': unittest.main()
