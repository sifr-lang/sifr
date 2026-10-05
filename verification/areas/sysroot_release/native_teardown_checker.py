"""Independent finite receipt replay for actual harmless production execution."""
import errno
import math
from pathlib import Path
import signal

import native_teardown_process as fixture


def require(condition, message):
    if not condition:
        raise ValueError(message)


def finite(value):
    return type(value) in (int, float) and math.isfinite(value)


def check_case(case, report):
    kind = case['kind']
    require(kind in fixture.KINDS, 'unknown fixture kind')
    expected_argv = [report['tools']['python']['path'], '-I', '-S', '-B',
        str(Path(report['source_root'])/'verification/areas/sysroot_release/native_teardown_process.py'),
        kind, str(Path(report['original_output'])/'temporary'/(kind+'-ready.json'))]
    require(case['argv'] == expected_argv, 'fixture command differs')
    expected_deadline = None if kind == fixture.KINDS[0] else fixture.DEADLINES[kind]
    require(case['deadline_seconds'] == expected_deadline, 'fixture deadline differs')
    events = case['events']
    last = 0
    allowed = {'spawn', 'snapshot', 'independent-snapshot', 'waitid', 'wait-start', 'wait-end',
               'failure-containment-wait', 'send', 'signal-start', 'signal-end',
               'members', 'cleanup-start', 'cleanup-end'}
    for event in events:
        require(event['kind'] in allowed and finite(event['time']) and
                last <= event['time'] <= report['finished'], 'observation order or kind differs')
        last = event['time']
        if event['kind'] in ('snapshot', 'independent-snapshot'):
            fixture.parse(event['raw'])
    # A partial exception receipt can be valid evidence but cannot pass a case.
    if case.get('failure') is not None:
        error = case['failure']
        require(isinstance(error, dict) and isinstance(error.get('type'), str)
                and isinstance(error.get('message'), str), 'invalid failure record')
        return False
    leader = case['leader']
    ready = case['ready']
    require(type(leader) is int and leader > 0 and type(ready.get('leader')) is int
            and ready['leader'] == leader and ready.get('kind') == kind, 'ready leader/kind differs')
    children = ready['children']
    count = 1 if kind == 'resistant-pipe' else 0
    require(isinstance(children, list) and len(children) == count and
            all(type(pid) is int and pid > 0 for pid in children) and
            len(set([leader, *children])) == count+1, 'ready children differ')
    spawned = [e for e in events if e['kind'] == 'spawn']
    require(len(spawned) == 1 and spawned[0]['pid'] == leader and
            spawned[0]['returncode'] is None, 'production owned spawn differs')
    driver = report['driver']
    members = None
    latest = None
    identities = {}
    reaped = False
    pending_wait = None
    pending_signal = None
    send = None
    signals = []
    term_finished = None
    errors = []
    held = []
    waitids = []
    waits = []
    cleanup_deadline = None
    complete = False
    independent_absence = False
    saw_live_child = False
    saw_dead_leader = False
    needs_dead_proof = False

    def rows(raw):
        nonlocal saw_live_child, saw_dead_leader
        parsed = fixture.parse(raw)
        selected = [r for r in parsed.values() if r['group'] == leader]
        for row in selected:
            pid = row['pid']
            require(pid in {leader, *children} and (not reaped or pid in identities),
                    'unknown fixture group member')
            identity = (row['start'], row['group'], row['uid'], row['ruid'])
            require(row['uid'] == driver['uid'] and row['ruid'] == driver['ruid'] and
                    (pid not in identities or identities[pid] == identity), 'owned identity changed')
            identities[pid] = identity
            if pid == leader:
                require(row['parent'] == driver['pid'], 'leader parent differs')
                saw_dead_leader |= row['state'].startswith('Z')
            else:
                require(row['parent'] in (leader, 1), 'child parent differs')
                saw_live_child |= not row['state'].startswith('Z')
        if not reaped:
            require(any(r['pid'] == leader for r in selected), 'unreaped leader anchor absent')
        else:
            require(all(r['pid'] != leader and r['state'].startswith('Z') for r in selected),
                    'live or changed group after reap')
        return selected

    for event in events:
        tag = event['kind']
        if tag in ('snapshot', 'independent-snapshot'):
            latest = rows(event['raw'])
            if tag == 'independent-snapshot':
                require(finite(event['started']) and 0 <= event['time']-event['started'] <= 1,
                        'independent probe time differs')
                if kind == 'waitid-capability' and not reaped:
                    held.append(event['label'])
                    require(len(latest) == 1 and latest[0]['state'].startswith('Z'),
                            'WNOWAIT did not retain owned zombie')
                if reaped:
                    require(not latest, 'final absence unproven')
                    independent_absence = True
        elif tag == 'members':
            require(kind != 'waitid-capability' and event['pid'] == leader and
                    type(event['reaped']) is bool and event['reaped'] == reaped and
                    event['rows'] == latest, 'production membership differs from raw snapshot')
            members = latest
            if needs_dead_proof:
                require(not any(not r['state'].startswith('Z') for r in members),
                        'signal error followed by live members')
                needs_dead_proof = False
        elif tag == 'send':
            require(not reaped and event['reaped'] is False and event['returncode'] is None
                    and event['pid'] == leader and event['rows'] == members and members
                    and any(not r['state'].startswith('Z') for r in members),
                    'signal decision lacks unreaped live ownership')
            require(send is None and event['signal'] in (signal.SIGTERM, signal.SIGKILL),
                    'invalid or duplicate signal decision')
            send = event['signal']
        elif tag == 'signal-start':
            require(not reaped and event['pid'] == leader and send == event['signal']
                    and pending_signal is None, 'actual signal lacks owned decision')
            if event['signal'] == signal.SIGKILL:
                require(term_finished is not None and event['time']-term_finished >= .25,
                        'KILL precedes production TERM grace')
            pending_signal = event
            send = None
        elif tag == 'signal-end':
            require(pending_signal is not None and event['pid'] == leader and
                    event['signal'] == pending_signal['signal'], 'signal result lacks actual call')
            signals.append(event['signal'])
            if event['signal'] == signal.SIGTERM:
                term_finished = event['time']
            if event['error'] is not None:
                require(event['error']['errno'] in (errno.EPERM, errno.ESRCH), 'unexpected signal error')
                errors.append(event['error'])
                needs_dead_proof = True
            pending_signal = None
        elif tag == 'wait-start':
            require(event['pid'] == leader and pending_wait is None and pending_signal is None
                    and not needs_dead_proof, 'wait order differs')
            if not reaped:
                observed = latest if kind == 'waitid-capability' else members
                require(event['returncode'] is None and observed is not None and
                        not any(not r['state'].startswith('Z') for r in observed) and saw_dead_leader,
                        'reap precedes dead-only proof')
                require(finite(event['kwargs'].get('timeout')) and event['kwargs']['timeout'] > 0,
                        'first reap lacks finite bound')
            else:
                require(event['returncode'] == waits[0], 'cached wait status changed')
            pending_wait = event
        elif tag == 'wait-end':
            require(pending_wait is not None and event['pid'] == leader and event['error'] is None
                    and type(event.get('returncode')) is int, 'actual wait failed or absent')
            waits.append(event['returncode'])
            require(all(value == waits[0] for value in waits), 'direct child wait status changed')
            reaped = True
            pending_wait = None
        elif tag == 'waitid':
            require(kind == 'waitid-capability' and not reaped and event['returncode'] is None
                    and event['pid'] == leader and type(event['idtype']) is int and event['idtype'] == 1
                    and event['flags'] == 0x25, 'WNOWAIT flags or ownership differs')
            value = event['value']
            require(isinstance(value, dict) and all(type(number) is int for number in value.values())
                    and value == dict(si_pid=leader, si_uid=driver['uid'], si_signo=20,
                                  si_status=7, si_code=1), 'WNOWAIT child status differs')
            waitids.append(value)
        elif tag == 'cleanup-start':
            require(kind != 'waitid-capability' and event['reaped'] == reaped,
                    'cleanup state differs')
        elif tag == 'cleanup-end':
            require(kind != 'waitid-capability' and event['error'] is None and
                    event['complete'] is True and event['reaped'] is True and reaped
                    and members == [] and event['signal_errors'] == errors,
                    'production cleanup proof or retained errors differs')
            deadline = event['deadline']
            require(finite(deadline) and event['time'] <= deadline+.5,
                    'production cleanup exceeded finite probe allowance')
            if cleanup_deadline is None:
                cleanup_deadline = deadline
            require(deadline == cleanup_deadline, 'cleanup retry reset deadline')
            complete = True
        elif tag == 'failure-containment-wait':
            raise ValueError('failure containment cannot pass')
    require(reaped and pending_wait is None and pending_signal is None and send is None
            and not needs_dead_proof and independent_absence and set(identities) == {leader, *children},
            'incomplete fixture ownership/reap/absence')
    if kind == 'waitid-capability':
        require([e['kind'] for e in events] == ['spawn', 'waitid', 'independent-snapshot',
                    'waitid', 'independent-snapshot', 'wait-start', 'wait-end', 'independent-snapshot'],
                'capability observation order differs')
        require(len(waitids) == 2 and held == ['held-zombie-0', 'held-zombie-1'] and
                waits == [7] and signals == [] and case['outcome'] is None,
                'capability sequence differs')
    else:
        outcome = case['outcome']
        cause, code, child_code = ('safety_deadline', 124, -signal.SIGTERM) if kind == 'deadline-term' else (
            'exit', 7 if kind == 'natural-output' else 0, 7 if kind == 'natural-output' else 0)
        require(complete and outcome['cause'] == cause and type(outcome['returncode']) is int
                and outcome['returncode'] == code and waits[0] == child_code
                and outcome['stdout'] == fixture.OUTPUT.hex() and outcome['stderr'] == fixture.ERROR_OUTPUT.hex()
                and outcome['truncated'] is False and finite(outcome['elapsed_seconds'])
                and 0 <= outcome['elapsed_seconds'] <= fixture.DEADLINES[kind]+5.5,
                'actual Outcome differs from fixed case')
        expected = [] if kind == 'natural-output' else ([signal.SIGTERM] if kind == 'deadline-term'
                                                      else [signal.SIGTERM, signal.SIGKILL])
        require(signals == expected and (kind != 'resistant-pipe' or saw_live_child),
                'required actual live-group signal sequence differs')
    return True
