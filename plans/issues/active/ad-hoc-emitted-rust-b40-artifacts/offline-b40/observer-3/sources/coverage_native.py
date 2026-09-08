"""B36 public Darwin sampled identity + continuous process-event custody.

No private PID-generation API, atomic table or check-and-signal claim. NOTE_EXIT
may be delayed under ptrace; absence comes from fresh ps AND native ESRCH.
"""
import copy
import ctypes
import errno
import os
import select
import time

NATIVE_SCHEMA = '12K-B36-public-native-v1'
STABLE = ('pid', 'pgid', 'start_sec', 'start_usec', 'path')
DEBUGGER = '/Applications/Xcode.app/Contents/Developer/usr/bin/lldb'
DEBUGSERVER = os.path.realpath('/Applications/Xcode.app/Contents/SharedFrameworks/LLDB.framework/Resources/debugserver')


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def stable(a, b):
    return all(k in a and k in b and a[k] == b[k] for k in STABLE)


class BSDInfo(ctypes.Structure):
    _fields_ = [(k, ctypes.c_uint32) for k in (
        'flags', 'status', 'xstatus', 'pid', 'ppid', 'uid', 'gid', 'ruid',
        'rgid', 'svuid', 'svgid', 'reserved')]
    _fields_ += [('comm', ctypes.c_char * 16), ('name', ctypes.c_char * 32)]
    _fields_ += [(k, ctypes.c_uint32) for k in ('nfiles', 'pgid', 'jobc', 'tdev', 'tpgid')]
    _fields_ += [('nice', ctypes.c_int32), ('start_sec', ctypes.c_uint64),
                ('start_usec', ctypes.c_uint64)]


class Darwin:
    """Instantiated only by the outer runner; imports/offline tests do no probing."""
    def __init__(self):
        self.lib = ctypes.CDLL('/usr/lib/libproc.dylib', use_errno=True)
        self.lib.proc_pidinfo.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.c_uint64,
                                         ctypes.c_void_p, ctypes.c_int]
        self.lib.proc_pidinfo.restype = ctypes.c_int
        self.lib.proc_pidpath.argtypes = [ctypes.c_int, ctypes.c_void_p, ctypes.c_uint32]
        self.lib.proc_pidpath.restype = ctypes.c_int
        self.queue = select.kqueue()

    def sample(self, pid):
        info = BSDInfo()
        ctypes.set_errno(0)
        count = self.lib.proc_pidinfo(pid, 3, 0, ctypes.byref(info), ctypes.sizeof(info))
        if count == 0 and ctypes.get_errno() == errno.ESRCH:
            return None
        require(count == ctypes.sizeof(info), 'native BSD acquisition failed/short: ' + str(pid))
        path = ctypes.create_string_buffer(4096)
        count = self.lib.proc_pidpath(pid, path, len(path))
        require(count > 0, 'native executable path acquisition failed: ' + str(pid))
        return {'pid': info.pid, 'ppid': info.ppid, 'pgid': info.pgid,
                'start_sec': info.start_sec, 'start_usec': info.start_usec,
                'path': os.fsdecode(path.value), 'status': info.status,
                'comm': os.fsdecode(info.comm)}

    def register(self, pid):
        event = select.kevent(pid, filter=select.KQ_FILTER_PROC,
            flags=select.KQ_EV_ADD | select.KQ_EV_ENABLE | 0x0040,  # public EV_RECEIPT
            fflags=select.KQ_NOTE_EXEC | select.KQ_NOTE_EXIT)
        response = self.queue.control([event], 1, 0)
        require(len(response) == 1 and response[0].ident == pid
                and response[0].flags & select.KQ_EV_ERROR and response[0].data == 0,
                'native watch registration failed')

    def poll(self):
        events = self.queue.control(None, 256, 0)
        require(len(events) < 256, 'native event bound reached')
        return [{'pid': e.ident, 'exec': bool(e.fflags & select.KQ_NOTE_EXEC),
                 'exit': bool(e.fflags & select.KQ_NOTE_EXIT),
                 'lost': bool(e.flags & select.KQ_EV_ERROR),
                 'eof': bool(e.flags & select.KQ_EV_EOF),
                 'flags': e.flags, 'fflags': e.fflags, 'data': e.data}
                for e in events]

    def close(self):
        self.queue.close()


class NativeCapture:
    def __init__(self, root, backend, ps_capture, monotonic=time.monotonic, inventory=None,
                 expected_binary=None):
        self.root, self.backend, self.ps_capture = root, backend, ps_capture
        self.monotonic = monotonic
        self.registrations, self.events = {}, {}
        self.error = None
        self.sequence = 0
        self.departed = set()
        self.inventory = inventory
        self.expected_binary = str(expected_binary) if expected_binary is not None else None

    def poll(self):
        for event in self.backend.poll():
            pid = event['pid']
            require(pid in self.registrations, 'unregistered native event')
            self.events[pid].append(copy.deepcopy(event))

    def __call__(self):
        from coverage_custody import ancestry, validate_snapshot, probe_row
        from coverage_acquisition import reconcile
        self.sequence += 1
        begin = self.monotonic()
        result = self.ps_capture()
        if self.inventory is not None:
            self.inventory.collect(result)
        result['native'] = {'schema': NATIVE_SCHEMA, 'sequence': self.sequence,
                            'begin': begin, 'records': {}, 'error': None}
        native = result['native']
        accepted_departed = set(self.departed)
        try:
            require(self.error is None, 'sticky native watch failure: ' + str(self.error))
            validate_snapshot(result)
            rows = {r['pid']: r for r in result['rows']}
            wanted = set(self.registrations) | {self.root} | {
                pid for pid in ancestry(result['rows'], self.root)
                if not probe_row(result, rows[pid])}
            require(len(wanted - self.departed) <= 64, 'native custody subject bound reached')
            self.poll()
            missing = []
            for pid in sorted(wanted):
                before = self.backend.sample(pid)
                entry = {'initial_sample': copy.deepcopy(before)}
                native['records'][str(pid)] = entry
                if before is None:
                    entry.update(present=False, absence='ESRCH')
                    require(pid != self.root, 'root disappearance ambiguity')
                    if pid in rows:
                        require(pid != self.root, 'root disappearance ambiguity')
                        missing.append(pid)
                    else:
                        self.departed.add(pid)
                    continue
                require(pid not in self.departed, 'departed PID returned during acquisition')
                require(pid in rows, 'native/ps presence ambiguity')
                if pid not in self.registrations:
                    self.backend.register(pid)
                    after = self.backend.sample(pid)
                    entry['registration'] = {'serial': len(self.registrations) + 1,
                        'sequence': self.sequence, 'before': before, 'after': after}
                    require(after is not None and stable(before, after)
                            and before['ppid'] == after['ppid'],
                            'registration bracket identity/parent changed')
                    self.registrations[pid] = {'serial': len(self.registrations) + 1,
                        'sequence': self.sequence, 'before': before, 'after': after}
                    self.events[pid] = []
                after = self.backend.sample(pid)
                entry.update(before=before, after=after)
                require(after is not None and stable(before, after), 'native bracket identity changed')
                entry.update(present=True, before=before, after=after)
            self.poll()
            for pid, registration in self.registrations.items():
                entry = native['records'][str(pid)]
                entry.update(registration=copy.deepcopy(registration),
                             events=copy.deepcopy(self.events[pid]))
            reconcile(self, result, missing)
            for pid in self.registrations:
                native['records'][str(pid)]['events'] = copy.deepcopy(self.events[pid])
            native['end'] = self.monotonic()
            from coverage_acquisition import validate_rounds
            validate_rounds(result)
        except Exception as exc:
            self.departed = accepted_departed
            self.error = str(exc)
            native['error'] = self.error
        native['end'] = self.monotonic()
        if self.inventory is not None:
            self.inventory.collect(result)
        return result

    def close(self):
        self.backend.close()


def checked_record(snapshot, pid, initial=None, allow_absent=False):
    import math
    native = snapshot.get('native', {})
    require(native.get('schema') == NATIVE_SCHEMA and native.get('error') is None,
            'native evidence missing or failed')
    require(type(native['sequence']) is int and native['sequence'] > 0,
            'invalid native sequence')
    require(all(type(native[k]) in (int, float) and math.isfinite(native[k])
                for k in ('begin', 'end')) and native['begin'] <= native['end'],
            'invalid native clock bracket')
    entry = native['records'].get(str(pid))
    require(entry is not None, 'native subject missing')
    registration = entry.get('registration')
    from coverage_acquisition import confirmed_departures
    corroborated = pid in confirmed_departures(snapshot)
    if registration is None and initial is None and corroborated:
        require(allow_absent, 'root absence not allowed')
        return entry
    if initial is not None:
        require(registration == initial['registration'], 'native watch registration replayed/replaced')
        prior = initial['events']
        require(entry['events'][:len(prior)] == prior, 'native event history lost')
    require(registration is not None and type(registration['serial']) is int
            and registration['serial'] > 0
            and 0 < registration['sequence'] <= native['sequence'], 'invalid native registration')
    require(stable(registration['before'], registration['after'])
            and registration['before']['ppid'] == registration['after']['ppid'],
            'contradictory native registration bracket')
    events = entry['events']
    require(isinstance(events, list) and all(e['pid'] == pid for e in events), 'wrong native events')
    require(not any(e['exec'] or e['lost'] or (e['eof'] and not e['exit']) for e in events),
            'native exec or watch loss')
    if not entry['present']:
        require(allow_absent and entry['absence'] == 'ESRCH', 'native subject absent')
        require(corroborated or not any(r['pid'] == pid for r in snapshot['rows']),
                'native/ps absence mismatch')
        return entry
    a, b = entry['before'], entry['after']
    require(stable(a, b) and stable(registration['after'], b), 'native identity/path changed')
    require(type(b['pid']) is int and b['pid'] == pid and type(b['pgid']) is int
            and b['pgid'] > 0 and type(b['start_sec']) is int and b['start_sec'] > 0
            and type(b['start_usec']) is int and 0 <= b['start_usec'] < 1000000
            and isinstance(b['path'], str) and b['path'].startswith('/'), 'invalid native identity')
    row = next((r for r in snapshot['rows'] if r['pid'] == pid), None)
    require(row is not None and row['pgid'] == b['pgid']
            and row['ppid'] in (a['ppid'], b['ppid']), 'native/ps parent/group mismatch')
    return entry


def live(entry):
    return (entry['present'] and entry['after']['status'] != 5
            and not any(e['exit'] or e['eof'] for e in entry['events']))
