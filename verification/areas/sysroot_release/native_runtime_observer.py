"""Sampled Darwin diagnostic memory stops; not a hard cap or escaped-tree custody."""
import json
import os
from pathlib import Path
import re
import selectors
import subprocess
import time

from native_capacity_diagnostics import parse_pages

SPEC = {'interval_seconds': .25, 'maximum_gap_seconds': 2.0,
        'rss_stop_bytes': 512*1024**2, 'available_stop_bytes': 9*1024**3//4,
        'probe_timeout_seconds': .5, 'probe_limit_bytes': 1024**2,
        'telemetry_limit_bytes': 128*1024**2}
PS = ['/bin/ps', '-axo', 'pid=,ppid=,pgid=,rss=,lstart=']
VM = ['/usr/bin/vm_stat']


def probe(argv):
    """Fixed small OS utilities only; never recursively enter execute's signals."""
    if argv not in (PS, VM):
        raise ValueError('undeclared observer probe')
    env = os.environ | {'LC_ALL': 'C'}
    proc = subprocess.Popen(argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env)
    streams = {'stdout': bytearray(), 'stderr': bytearray()}
    end = time.monotonic()+SPEC['probe_timeout_seconds']
    try:
        with selectors.DefaultSelector() as select:
            for name in streams:
                select.register(getattr(proc, name), selectors.EVENT_READ, name)
            while select.get_map():
                if time.monotonic() >= end:
                    raise ValueError('observer probe deadline')
                for key, _ in select.select(min(.02, max(0, end-time.monotonic()))):
                    data = os.read(key.fd, 65536)
                    if not data:
                        select.unregister(key.fileobj)
                        continue
                    streams[key.data].extend(data)
                    if sum(map(len, streams.values())) > SPEC['probe_limit_bytes']:
                        raise ValueError('observer probe output limit')
            code = proc.wait(timeout=max(.001, end-time.monotonic()))
        if code or streams['stderr']:
            raise ValueError('observer probe failed')
        return streams['stdout'].decode('utf-8', errors='strict'), proc.pid
    finally:
        if proc.poll() is None:
            proc.kill()
        proc.wait()
        proc.stdout.close()
        proc.stderr.close()


def parse_process_rows(raw):
    rows = {}
    for line in raw.splitlines():
        match = re.fullmatch(r'\s*(\d+)\s+(\d+)\s+(\d+)\s+(\d+)\s+([A-Za-z]{3} [A-Za-z]{3}\s+\d{1,2} \d\d:\d\d:\d\d \d{4})\s*', line)
        if not match:
            raise ValueError('invalid Darwin ps row')
        pid, parent, group, rss = map(int, match.groups()[:4])
        if pid < 0 or group < 0 or pid in rows:
            raise ValueError('duplicate or invalid Darwin PID')
        rows[pid] = {'pid': pid, 'ppid': parent, 'pgid': group,
                     'rss_bytes': rss*1024, 'start': match[5]}
    if not rows:
        raise ValueError('empty Darwin process observation')
    return rows


def interpret(row, total):
    if any(type(row[name]) is not int or row[name] <= 0
           for name in ('driver_pid', 'collector_pid', 'pgid')):
        raise ValueError('owned diagnostic process identities must be positive integers')
    rows = parse_process_rows(row['ps'])
    if row['driver_pid'] not in rows or row['collector_pid'] not in rows:
        raise ValueError('driver or observer missing')
    # A system PID-0 row may be present in a host inventory, but cannot be an
    # owned command, driver or collector, even if its reported group matches.
    members = sorted(pid for pid, value in rows.items() if pid > 0 and
                     (value['pgid'] == row['pgid'] or pid in (row['driver_pid'], row['collector_pid'])))
    pages = parse_pages(row['vm_stat'])
    if not 0 <= pages['available_bytes'] <= total:
        raise ValueError('invalid observed memory availability')
    rss = sum(rows[pid]['rss_bytes'] for pid in members)
    stop = ('observer_memory' if rss >= SPEC['rss_stop_bytes'] else
            'observer_reserve' if pages['available_bytes'] <= SPEC['available_stop_bytes'] else None)
    return {'members': [rows[pid] for pid in members], 'rss_bytes': rss,
            'available_bytes': pages['available_bytes'], 'stop': stop,
            'leader_observed': row['pgid'] in rows}


class Observer:
    def __init__(self, path, *, phase, total, collect=probe, clock=time.monotonic):
        self.path, self.phase, self.total = Path(path), phase, total
        self.collect, self.clock = collect, clock
        self.last = None
        self.last_pid = None
        self.samples = 0
        self.maximum = 0
        self.stop = None
        self.identities = {}

    def poll(self, pid, now):
        if self.last is not None and pid == self.last_pid and now-self.last < SPEC['interval_seconds']:
            return None
        row = {'phase': self.phase, 'pgid': pid, 'driver_pid': os.getpid(), 'started': now}
        previous_maximum = self.maximum
        try:
            if self.last is not None and now-self.last > SPEC['maximum_gap_seconds']:
                raise ValueError('observer sample stale')
            row['ps'], row['collector_pid'] = self.collect(PS)
            row['vm_stat'], _ = self.collect(VM)
            row['finished'] = self.clock()
            if not 0 <= row['finished']-now <= SPEC['maximum_gap_seconds']:
                raise ValueError('observer collection stale')
            row['observation'] = interpret(row, self.total)
            for member in row['observation']['members']:
                identity = (member['start'], member['pgid'])
                previous = self.identities.setdefault(member['pid'], identity)
                if previous != identity:
                    raise ValueError('observed PID identity changed')
            self.stop = row['observation']['stop']
            self.maximum = max(self.maximum, row['observation']['rss_bytes'])
        except (ValueError, OSError, subprocess.SubprocessError, KeyError) as error:
            row['error'] = type(error).__name__+': '+str(error)
            self.stop = 'observer_unavailable'
        row['stop'] = self.stop
        encoded = (json.dumps(row, sort_keys=True)+'\n').encode()
        if self.path.exists() and self.path.stat().st_size+len(encoded) > SPEC['telemetry_limit_bytes']-1024:
            self.stop = 'observer_evidence_limit'
            self.maximum = previous_maximum
            row = {'phase': self.phase, 'started': now, 'pgid': pid, 'driver_pid': os.getpid(),
                   'error': 'telemetry allowance exhausted', 'stop': self.stop}
            encoded = (json.dumps(row, sort_keys=True)+'\n').encode()
        with self.path.open('ab') as stream:
            stream.write(encoded)
        self.last = now
        self.last_pid = pid
        self.samples += 1
        return self.stop

    def summary(self):
        return {'samples': self.samples, 'sampled_maximum_rss_bytes': self.maximum, 'stop': self.stop,
                'coverage': 'sampled-session-group-and-driver; between-sample peaks and setsid escapes unbounded'}


def replay(path, total):
    phases = {}
    identities = {}
    if path.stat().st_size > SPEC['telemetry_limit_bytes']:
        raise ValueError('telemetry allowance exceeded')
    with path.open('rb') as stream:
        for line in stream:
            if len(line) > 2*SPEC['probe_limit_bytes']+65536:
                raise ValueError('telemetry record exceeds bound')
            row = json.loads(line)
            phase = phases.setdefault(row['phase'], {'samples': 0, 'sampled_maximum_rss_bytes': 0,
                'stop': None, 'last': None})
            if phase['stop'] is not None:
                raise ValueError('samples follow terminal stop')
            if 'error' not in row:
                observation = interpret(row, total)
                if row['observation'] != observation or row['stop'] != observation['stop']:
                    raise ValueError('sample arithmetic differs')
                if not 0 <= row['finished']-row['started'] <= SPEC['maximum_gap_seconds']:
                    raise ValueError('sample duration differs')
                if phase['last'] is not None and not 0 <= row['started']-phase['last'] <= SPEC['maximum_gap_seconds']:
                    raise ValueError('sample gap differs')
                for member in observation['members']:
                    key = (row['phase'], member['pid'])
                    value = (member['start'], member['pgid'])
                    if identities.setdefault(key, value) != value:
                        raise ValueError('retained PID identity changed')
                phase['sampled_maximum_rss_bytes'] = max(phase['sampled_maximum_rss_bytes'], observation['rss_bytes'])
            elif row['stop'] not in ('observer_unavailable', 'observer_evidence_limit'):
                raise ValueError('invalid telemetry failure')
            phase['last'] = row['started']
            phase['samples'] += 1
            phase['stop'] = row['stop']
    for phase in phases.values():
        del phase['last']
    return phases
