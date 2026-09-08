"""Synthetic native evidence only. Never asserts any historical process identity."""
import copy
import os
from coverage_native import NATIVE_SCHEMA, DEBUGGER


def enrich(snapshot, prior=None):
    records = {}
    old = prior['native']['records'] if prior and 'native' in prior else {}
    tick = snapshot['begin_monotonic']
    for row in snapshot['rows']:
        pid = row['pid']
        path = DEBUGGER if row['command'] == '/synthetic/lldb' else row['command']
        if path.startswith('/Applications/Xcode.app/'):
            path = os.path.realpath(path)  # explicitly synthetic native path, raw ps untouched
        sample = {'pid': pid, 'ppid': row['ppid'], 'pgid': row['pgid'],
                  'start_sec': 1000, 'start_usec': pid, 'path': path, 'status': 3}
        sample['comm'] = path.rsplit('/', 1)[-1][:16]
        registration = {'serial': pid, 'sequence': 1,
                        'before': copy.deepcopy(sample), 'after': copy.deepcopy(sample)}
        entry = old.get(str(pid))
        if entry:
            registration = copy.deepcopy(entry['registration'])
        records[str(pid)] = {'present': True, 'before': copy.deepcopy(sample),
            'after': copy.deepcopy(sample), 'registration': registration,
            'events': copy.deepcopy(entry['events']) if entry else []}
    for pid, entry in old.items():
        if pid not in records:
            records[pid] = dict(copy.deepcopy(entry), present=False, absence='ESRCH')
    snapshot['native'] = {'schema': NATIVE_SCHEMA, 'sequence': int(tick),
        'begin': tick, 'end': tick + .01, 'error': None, 'records': records}
    return snapshot
