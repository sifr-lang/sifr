"""Synthetic producer/consumer replay; no inferior or debugger is launched."""
import copy
import json
import os
import tempfile
from pathlib import Path
from coverage_output import (RUN_ID, STREAMS, configure_launch, create_route,
                             finalize, parse_completion)
from coverage_symbols import FAMILIES, require

GOOD = (b'format check passed\n'
        b'B20_PHASE constructor 1 2 3 4\n'
        b'B20_PHASE file.before.read 10 20 30 40\n'
        b'B20_PHASE file.after.read 11 21 31 41\n'
        b'B20_PHASE file.after.check 12 22 32 42\n'
        b'B20_PHASE file.before.read 13 23 33 43\n'
        b'B20_PHASE file.after.read 14 24 34 44\n'
        b'B20_PHASE file.after.check 15 25 35 45\n'
        b'B20_PHASE cli.done 16 26 36 46\n')


class SyntheticLaunch:
    def __init__(self, reject_fd=None):
        self.actions = []
        self.reject_fd = reject_fd

    def AddOpenFileAction(self, fd, path, read, write):
        self.actions.append((fd, path, read, write))
        return fd != self.reject_fd

    def produce(self, stdout, stderr):
        for fd, path, read, write in self.actions:
            require(read is False and write is True, 'wrong synthetic action')
            # Mimic the pinned O_WRONLY|O_TRUNC action on the exclusive inode.
            with open(path, 'wb') as stream:
                stream.write(stdout if fd == 1 else stderr)


def run_tests(results):
    def check(name, action, expected=True):
        try:
            action()
            accepted, error = True, None
        except (ValueError, KeyError, TypeError, IndexError, OSError) as exc:
            accepted, error = False, str(exc)
        results.append({'name': 'output-' + name, 'expected_accept': expected,
                        'accepted': accepted, 'pass': accepted == expected, 'error': error})

    def context(stdout=b'', stderr=GOOD):
        root = Path(tempfile.mkdtemp(prefix='b32-synthetic-output-')).resolve()
        binary = root / 'synthetic-not-an-inferior'
        route = create_route(root, RUN_ID, binary)
        launch = SyntheticLaunch()
        actions = configure_launch(launch, route, root, RUN_ID, binary)
        launch.produce(stdout, stderr)
        from coverage_custody_tests import synthetic_context
        custodian, handshake, ack = synthetic_context(binary)
        observer = {'state': 'ENTRIES_COMPLETE', 'stage': 'completion', 'run_id': RUN_ID,
                    'pid': 123, 'target_launches': 1, 'output_actions': actions,
                    'exit': 0, 'exited': True, 'final_phase': 'active', 'handover_count': 1,
                    'hits': ['prepare', 'fixups', 'generic', 'libsystem', 'sanitizers',
                             'constructor', 'main', 'completion']}
        from coverage_custody_tests import snapshot
        custodian.observe(snapshot([], 20, prior=custodian.last))
        custody = custodian.document()
        release = {'remaining': [], 'groups_absent': True, 'watcher_alive': False,
                   'monitor_alive': False}
        return [route, observer, handshake, ack, custody, release]

    def success():
        receipt = finalize(*context())
        require(receipt['state'] == 'PASS' and receipt['file_check_phases'] == 2
                and receipt['finalized_after_exit_and_release'], 'incomplete final receipt')
        require(receipt['streams']['stdout']['size'] == 0, 'wrong stdout role')
    check('complete-producer-consumer-protocol-decision', success)
    negative_streams = [
        ('empty', b'', b''),
        ('truncated-last-line', b'', GOOD[:-1]),
        ('truncated-payload', b'', GOOD.replace(b'12 22 32 42', b'12 22 32')),
        ('one-file', b'', GOOD.replace(b'B20_PHASE file.after.check 15 25 35 45\n', b'')),
        ('three-file-checks', b'', GOOD.replace(b'B20_PHASE cli.done', b'B20_PHASE file.after.check 17 27 37 47\nB20_PHASE cli.done')),
        ('duplicate-record', b'', GOOD.replace(b'15 25 35 45', b'12 22 32 42')),
        ('duplicate-cli', b'', GOOD + b'B20_PHASE cli.done 16 26 36 46\n'),
        ('no-cli', b'', GOOD.replace(b'B20_PHASE cli.done 16 26 36 46\n', b'')),
        ('wrong-count-field', b'', GOOD.replace(b'12 22 32 42', b'12 22 32 42 52')),
        ('nondecimal', b'', GOOD.replace(b'12 22 32 42', b'12 22 x 42')),
        ('negative-payload', b'', GOOD.replace(b'12 22 32 42', b'-12 22 32 42')),
        ('debugger-echo', b'', GOOD.replace(b'B20_PHASE file.after.check 12', b'(lldb) B20_PHASE file.after.check 12')),
        ('substring-marker', b'', GOOD.replace(b'B20_PHASE file.after.check 12', b'echo B20_PHASE file.after.check 12')),
        ('wrong-stream', GOOD, b''),
        ('duplicate-cross-stream', GOOD, GOOD),
        ('missing-success', b'', GOOD.replace(b'format check passed\n', b'')),
        ('duplicate-success', b'', b'format check passed\n' + GOOD),
        ('wrong-file-order', b'', GOOD.replace(b'file.after.read', b'file.before.read')),
    ]
    for name, stdout, stderr in negative_streams:
        check(name, lambda stdout=stdout, stderr=stderr: finalize(*context(stdout, stderr)), False)
    mutations = [
        ('wrong-run-observer', lambda c: c[1].update(run_id='old')),
        ('wrong-run-route', lambda c: c[0].update(run_id='old')),
        ('wrong-run-handshake', lambda c: c[2].update(run_id='old')),
        ('wrong-pid', lambda c: c[1].update(pid=456)),
        ('two-targets', lambda c: c[4].update(target_ids=[123, 456])),
        ('wrong-binary', lambda c: c[0].update(binary='/wrong')),
        ('wrong-group', lambda c: c[2].update(pgid=456)),
        ('wrong-flags', lambda c: c[2].update(flags=2)),
        ('missing-custody', lambda c: c[4].update(processes=[])),
        ('wrong-actions', lambda c: c[1].update(output_actions=[])),
        ('observer-error', lambda c: c[1].update(error='failed')),
        ('observer-not-complete', lambda c: c[1].update(state='PASS')),
        ('nonzero-exit', lambda c: c[1].update(exit=1)),
        ('not-exited', lambda c: c[1].update(exited=False)),
        ('remaining-process', lambda c: c[5].update(remaining=[123])),
        ('live-group', lambda c: c[5].update(groups_absent=False)),
        ('live-watcher', lambda c: c[5].update(watcher_alive=True)),
        ('live-monitor', lambda c: c[5].update(monitor_alive=True)),
        ('missing-handover', lambda c: c[1].update(handover_count=0)),
        ('missing-entry', lambda c: c[1]['hits'].pop()),
        ('wrong-protocol-phase', lambda c: c[1].update(final_phase='handover')),
        ('stale-inode', lambda c: c[0]['streams']['stderr'].update(inode=0)),
    ]
    for name, mutate in mutations:
        def action(mutate=mutate):
            c = context()
            mutate(c)
            finalize(*c)
        check(name, action, False)

    for fd in (1, 2):
        def rejected_action(fd=fd):
            root = Path(tempfile.mkdtemp(prefix='b32-synthetic-actions-')).resolve()
            route = create_route(root, RUN_ID, root / 'fake')
            configure_launch(SyntheticLaunch(fd), route, root, RUN_ID, root / 'fake')
        check('unsupported-fd-' + str(fd), rejected_action, False)

    def stale_prelaunch():
        c = context()
        root = Path(c[0]['streams']['stdout']['path']).parent
        configure_launch(SyntheticLaunch(), c[0], root, RUN_ID, c[0]['binary'])
    check('nonempty-prelaunch', stale_prelaunch, False)

    def existing_route():
        c = context()
        root = Path(c[0]['streams']['stdout']['path']).parent
        create_route(root, RUN_ID, c[0]['binary'])
    check('existing-route-no-retry', existing_route, False)

    def debugger_only():
        c = context(b'', b'')
        c[1]['debugger_stdout'] = GOOD.decode('ascii')
        finalize(*c)
    check('debugger-only-markers-ignored', debugger_only, False)

    # Historical bytes are parser samples only, never retroactive B31 acceptance.
    old = Path('/private/tmp/sifr-b40.urwpgv/evidence/inputs/b31-debugger-result.json')
    transcript = json.loads(old.read_text())['stdout']
    check('whole-historical-debugger-transcript-rejected',
          lambda: parse_completion(b'', transcript.encode()), False)
    historical_lines = [line for line in transcript.splitlines()
                        if line == 'format check passed' or line.startswith('B20_PHASE ')]
    historical_sample = ('\n'.join(historical_lines) + '\n').encode()
    check('historical-payload-syntax-only', lambda: parse_completion(b'', historical_sample))
    check('historical-observer-failure-stays-rejected', lambda: finalize(
        context()[0], json.loads((old.parent / 'b31-observer-result.json').read_text()),
        *context()[2:]), False)
