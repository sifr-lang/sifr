"""Tiny process and retained-parser controls; no native workload."""
import json
import os
from pathlib import Path
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT/'verification/runner'))
from sifr_verify.process_execution import execute
import native_runtime_observer as observer

PAGES = ('Mach Virtual Memory Statistics: (page size of 16384 bytes)\n'
         'Pages free: 196608.\nPages inactive: 0.\nPages speculative: 0.\n'
         'Pages occupied by compressor: 0.\nSwapins: 0.\nSwapouts: 0.\n')


def raw(pid, group, rss=100):
    return f'{pid} 1 {group} {rss} Sun Oct  4 12:00:00 2026\n'


class ObserverTests(unittest.TestCase):
    def test_group_plus_driver_and_collector_without_double_count(self):
        row = {'ps': raw(1, 4)+raw(2, 4)+raw(3, 5)+raw(8, 8, 999999),
               'vm_stat': PAGES, 'pgid': 4, 'driver_pid': 1, 'collector_pid': 3}
        result = observer.interpret(row, 7*1024**3)
        self.assertEqual(result['rss_bytes'], 300*1024)
        self.assertIsNone(result['stop'])
        row['ps'] = raw(1, 4, 512*1024)+raw(3, 5)
        self.assertEqual(observer.interpret(row, 7*1024**3)['stop'], 'observer_memory')
        row['ps'] = raw(1, 4)+raw(3, 5)
        row['vm_stat'] = PAGES.replace('196608', '147456')
        self.assertEqual(observer.interpret(row, 7*1024**3)['stop'], 'observer_reserve')

    def test_bad_rows_and_contradictory_memory_reject(self):
        for text in ('', '1 1 1 -1 today', raw(1, 1)*2, raw(0, 1)):
            with self.subTest(text=text), self.assertRaises(ValueError):
                observer.parse_process_rows(text)
        with self.assertRaises(ValueError):
            observer.interpret({'ps': raw(1, 1), 'vm_stat': PAGES, 'pgid': 1,
                                'driver_pid': 1, 'collector_pid': 1}, 1024)

    def test_replay_recomputes_and_rejects_tamper(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary)/'samples'
            row = {'phase': 'unit', 'started': 1., 'finished': 1.1, 'pgid': 1,
                   'driver_pid': 1, 'collector_pid': 2, 'ps': raw(1, 1)+raw(2, 2), 'vm_stat': PAGES}
            row['observation'] = observer.interpret(row, 7*1024**3); row['stop'] = None
            path.write_text(json.dumps(row)+'\n')
            self.assertEqual(observer.replay(path, 7*1024**3)['unit']['samples'], 1)
            row['observation']['rss_bytes'] += 1
            path.write_text(json.dumps(row)+'\n')
            with self.assertRaises(ValueError): observer.replay(path, 7*1024**3)

    def test_missing_telemetry_returns_stop_and_retains_error(self):
        with tempfile.TemporaryDirectory() as temporary:
            def fail(_): raise ValueError('missing')
            item = observer.Observer(Path(temporary)/'samples', phase='unit', total=7*1024**3, collect=fail)
            self.assertEqual(item.poll(123, time.monotonic()), 'observer_unavailable')
            self.assertEqual(observer.replay(item.path, item.total)['unit']['stop'], 'observer_unavailable')

    def test_observer_stop_preserves_output_and_kills_group(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); ready = root/'ready'; escaped = root/'escaped'
            code = ("import os,time\nfrom pathlib import Path\n"
                    "if os.fork()==0:\n"
                    f" Path({str(ready)!r}).write_text('ready')\n"
                    " time.sleep(.7)\n"
                    f" Path({str(escaped)!r}).write_text('escaped')\n"
                    "else:\n print('kept',flush=True)\n time.sleep(5)\n")
            seen = time.monotonic()
            def poll(pid, now):
                return 'observer_memory' if ready.exists() and now-seen > .15 else None
            result = execute([sys.executable, '-B', '-c', code], cwd=root, observer=poll, deadline_seconds=3)
            self.assertEqual(result.cause, 'observer_memory')
            self.assertIn(b'kept', result.stdout)
            time.sleep(.75)
            self.assertFalse(escaped.exists())

    def test_observer_exception_cleanup_and_invalid_stop(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for value in ('invented',):
                with self.assertRaises(ValueError):
                    execute([sys.executable, '-B', '-c', 'import time; time.sleep(5)'],
                            cwd=root, observer=lambda *_: value)
            def fail(*_): raise RuntimeError('observer failure')
            with self.assertRaisesRegex(RuntimeError, 'observer failure'):
                execute([sys.executable, '-B', '-c', 'import time; time.sleep(5)'], cwd=root, observer=fail)

    def test_default_executor_unchanged(self):
        value = execute([sys.executable, '-B', '-c', "print('plain')"], cwd=ROOT)
        self.assertEqual((value.cause, value.returncode, value.stdout), ('exit', 0, b'plain\n'))

    def test_actual_fixed_ps_collector_and_parser(self):
        text, pid = observer.probe(observer.PS)
        rows = observer.parse_process_rows(text)
        self.assertIn(os.getpid(), rows)
        self.assertIn(pid, rows)

    def test_telemetry_limit_keeps_a_replayable_terminal_marker(self):
        with tempfile.TemporaryDirectory() as temporary, patch.dict(observer.SPEC, telemetry_limit_bytes=2048):
            def collect(argv):
                return (raw(os.getpid(), 77)+raw(123456, 99), 123456) if argv == observer.PS else (PAGES, 999)
            clock = [1.1]
            item = observer.Observer(Path(temporary)/'samples', phase='unit', total=7*1024**3,
                                     collect=collect, clock=lambda: clock[0])
            self.assertIsNone(item.poll(77, 1.))
            clock[0] = 1.6
            self.assertEqual(item.poll(77, 1.5), 'observer_evidence_limit')
            self.assertLessEqual(item.path.stat().st_size, 2048)
            summary = item.summary()
            self.assertEqual(observer.replay(item.path, item.total)['unit'],
                {name: summary[name] for name in ('samples', 'sampled_maximum_rss_bytes', 'stop')})

    def test_stale_and_pid_reuse_are_terminal(self):
        for change in ('stale', 'identity'):
            with tempfile.TemporaryDirectory() as temporary:
                stamp = ['12:00:00']
                def collect(argv):
                    return ((raw(os.getpid(), 77)+raw(123456, 99)).replace('12:00:00', stamp[0]), 123456) if argv == observer.PS else (PAGES, 999)
                clock = [1.1]
                item = observer.Observer(Path(temporary)/'samples', phase='unit', total=7*1024**3,
                                         collect=collect, clock=lambda: clock[0])
                self.assertIsNone(item.poll(77, 1.))
                if change == 'identity': stamp[0] = '12:01:00'
                clock[0] = 4.1 if change == 'stale' else 1.6
                self.assertEqual(item.poll(77, 4. if change == 'stale' else 1.5), 'observer_unavailable')


if __name__ == '__main__': unittest.main()
