"""Synthetic diagnostic controls; no Darwin runtime qualification is claimed."""
import copy
import json
from pathlib import Path
import plistlib
import tempfile
import unittest
from unittest.mock import patch

import native_capacity_diagnostics as probe
from sifr_verify.process_execution import Outcome


class DiagnosticControls(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.output = Path(self.temp.name) / 'observation'
        self.identity = {'commit': 'a' * 40,
                         'producer_sha256': {p: probe.digest(probe.ROOT / p) for p in probe.PRODUCERS}}
        self.pages = ('Mach Virtual Memory Statistics: (page size of 16384 bytes)\n'
                      'Pages free: 5135.\nPages inactive: 167577.\nPages speculative: 23898.\n'
                      'Pages occupied by compressor: 22402.\nSwapins: 0.\nSwapouts: 0.\n')
        self.mutation = None

    def command(self, argv, **kwargs):
        argv = [Path(argv[0]).name, *argv[1:]]
        if argv[:3] == ['df', '-k', '-P']:
            raw = 'Filesystem 1024-blocks Used Available Capacity Mounted on\n/dev/disk3s1 300000000 1000 299999000 1% /System/Volumes/Data\n'
        elif argv == ['diskutil', 'info', '-plist', '/dev/disk3s1']:
            raw = plistlib.dumps({'DeviceNode': '/dev/disk3s1', 'FilesystemType': 'apfs',
                                 'APFSPhysicalStores': [{'APFSPhysicalStore': 'disk0s2'}]})
        elif argv == ['diskutil', 'info', '-plist', '/dev/disk0s2']:
            raw = plistlib.dumps({'DeviceNode': '/dev/disk0s2', 'BusProtocol': 'Apple Fabric'})
        else:
            raw = {tuple(a): {
                'host': 'Darwin arm64\n', 'version': 'ProductVersion: 15.6\n',
                'memory-total': str(7 * 1024**3) + '\n', 'cpu': '4\n', 'pages': self.pages,
                'swap': 'vm.swapusage: total = 0.00M used = 0.00M free = 0.00M\n',
                'pressure': 'System-wide memory free percentage: 43%\n',
            }[name] for name, a in probe.FIXED}[tuple(argv)]
        if isinstance(raw, str):
            raw = raw.encode()
        return Outcome(2 if self.mutation == 'failure' and argv == ['memory_pressure', '-Q'] else 0,
                       'exit', raw, b'', self.mutation == 'truncated' and argv == ['vm_stat'], .01)

    def observe(self):
        with patch.object(probe.platform, 'system', return_value='Darwin'), \
                patch.object(probe.platform, 'python_version', return_value='3.14.7'), \
                patch.object(probe, 'source_identity', return_value=self.identity), \
                patch.object(probe, 'execute', side_effect=self.command), \
                patch.object(probe, 'selected_executable', side_effect=lambda name: {'path': '/fake/bin/' + name, 'sha256': 'f' * 64}), \
                patch.dict(probe.os.environ, {'SIFR_NATIVE_HOST_KIND': 'dedicated-darwin'}):
            return probe.observe(probe.ROOT, self.output, target='aarch64-apple-darwin')

    def report(self):
        return json.loads((self.output / 'state.json').read_text())

    def replace(self, report):
        for name in ('state.json', 'receipt.json'):
            (self.output / name).write_text(json.dumps(report))

    def test_retained_pages_recompute_available_and_do_not_claim_native_qualification(self):
        record = self.observe()
        self.assertEqual(record['observations']['memory']['available_bytes'], 3221258240)
        self.assertEqual(record['observations']['memory']['compressed_physical_bytes'], 367034368)
        self.assertEqual(record['runtime_assertions'], 0)
        self.assertIs(record['native_qualification'], False)
        self.assertEqual(probe.check_retained(self.output, 'a' * 40), record)
        moved = Path(self.temp.name) / 'retained-copy'
        import shutil
        shutil.copytree(self.output, moved)
        self.assertEqual(probe.check_retained(moved, 'a' * 40), record)

    def test_required_failed_truncated_or_incomplete_observations_retain_failure(self):
        for mutation in ('failure', 'truncated', 'missing-pages'):
            with self.subTest(mutation=mutation):
                self.output = Path(self.temp.name) / mutation
                self.mutation = mutation
                if mutation == 'missing-pages':
                    self.pages = self.pages.replace('Swapouts: 0.\n', '')
                with self.assertRaises(ValueError):
                    self.observe()
                self.assertEqual(self.report()['status'], 'failed')
                self.assertFalse((self.output / 'receipt.json').exists())
                with self.assertRaises(ValueError):
                    probe.check_retained(self.output, 'a' * 40)

    def test_raw_byte_hash_and_symlink_substitution_are_rejected(self):
        self.observe()
        raw = self.output / 'pages.stdout'
        data = raw.read_bytes(); raw.write_bytes(data + b' ')
        with self.assertRaisesRegex(ValueError, 'raw bytes'):
            probe.check_retained(self.output, 'a' * 40)
        raw.unlink(); target = Path(self.temp.name) / 'substitute'; target.write_bytes(data)
        raw.symlink_to(target)
        with self.assertRaisesRegex(ValueError, 'raw bytes'):
            probe.check_retained(self.output, 'a' * 40)

    def test_commit_producer_command_and_observation_drift_are_rejected(self):
        self.observe(); original = self.report()
        for mutation in ('source', 'producer', 'command', 'observation', 'duplicate', 'missing'):
            report = copy.deepcopy(original)
            if mutation == 'source': report['source']['commit'] = 'b' * 40
            if mutation == 'producer': report['source']['producer_sha256'].pop(probe.PRODUCERS[0])
            if mutation == 'command': report['commands'][-1]['argv'] = ['cargo', 'build']
            if mutation == 'observation': report['observations']['memory']['available_bytes'] += 1
            if mutation == 'duplicate': report['commands'].append(report['commands'][-1])
            if mutation == 'missing': report['commands'].pop()
            self.replace(report)
            with self.subTest(mutation=mutation), self.assertRaises((ValueError, KeyError)):
                probe.check_retained(self.output, 'a' * 40)

    def test_incomplete_or_forged_qualification_claims_are_rejected(self):
        self.observe(); original = self.report()
        for field, values in (('runtime_assertions', (True, 1)), ('native_qualification', (True, 0)),
                              ('status', ('incomplete', 'failed')), ('claim', ('native-qualified',))):
            for value in values:
                report = copy.deepcopy(original); report[field] = value; self.replace(report)
                with self.subTest(field=field, value=value), self.assertRaises(ValueError):
                    probe.check_retained(self.output, 'a' * 40)
        self.replace(original)
        with self.assertRaises(ValueError):
            probe.check_retained(self.output, 'a' * 40, target='x86_64-apple-darwin')

    def test_live_checker_rejects_interpreter_change_and_receipt_divergence(self):
        self.observe()
        with patch.object(probe, 'source_identity', return_value=self.identity), \
                patch.object(probe.sys, 'executable', str(Path(self.temp.name) / 'wrong-python')):
            with self.assertRaises(OSError):
                probe.check(self.output)
        (self.output / 'receipt.json').write_text('{}')
        with self.assertRaisesRegex(ValueError, 'receipt differs'):
            probe.check_retained(self.output, 'a' * 40)


if __name__ == '__main__':
    unittest.main()
