"""Pure boundary identity/pairing, local clocks and SDK counter layout."""
import ctypes
import hashlib
import json
import math
import re
from pathlib import Path
from coverage_symbols import require, uint

E = Path(__file__).resolve().parent
ROOT = E.parent / 'sifr'
BINARY = E / 'sifr-experiment09'
OUT = E / 'boundaries'
RETURN_FAMILIES = frozenset(('prepare', 'fixups', 'generic', 'libsystem', 'sanitizers'))
METRICS = ('ri_instructions', 'ri_cycles', 'ri_user_time', 'ri_system_time',
           'ri_pageins', 'ri_diskio_bytesread', 'ri_diskio_byteswritten')
SDK = Path('/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk/usr/include/sys/resource.h')

def run_id(index):
    require(type(index) is int and 0 <= index < 6, 'target schedule exhausted')
    return '12K-B40-urwpgv-target-' + str(index)

def save(path, value):
    path = Path(path)
    temp = path.with_suffix('.tmp')
    temp.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')
    temp.replace(path)

def read(path):
    path = Path(path)
    return json.loads(path.read_text()) if path.exists() else None

def elapsed(now, origin, limit):
    require(all(type(x) in (int, float) and math.isfinite(x) for x in (now, origin, limit)), 'invalid clock')
    require(0 <= now - origin < limit, 'local elapsed deadline')
    return now - origin

def full_target_fits(now, outer_start):
    elapsed(now, outer_start, 510)
    require(now - outer_start + 60 <= 510, 'full60s target cannot fit before cleanup')

def preflight_count(label, count):
    require(type(count) is int and 0 <= count <= 1, 'ambiguous prelaunch symbol locations')
    if label in ('constructor', 'main', 'completion'):
        require(count == 1, 'missing prelaunch application symbol')

def sdk_layout():
    raw = SDK.read_bytes()
    declaration = raw.decode().split('struct rusage_info_v4 {')[1].split('};')[0]
    lines = [s.strip() for s in declaration.splitlines() if s.strip()]
    require(lines[0] == 'uint8_t  ri_uuid[16];', 'SDK UUID layout changed')
    fields = re.findall(r'uint64_t (\w+);', declaration)
    require(len(fields) == 35 and len(lines) == 36 and set(METRICS) <= set(fields), 'SDK v4 layout changed')
    units = {}
    for field in fields:
        if field in ('ri_instructions', 'ri_cycles'):
            unit = field.removeprefix('ri_') if hasattr(field, 'removeprefix') else field[3:]
        elif 'pageins' in field or 'wkups' in field:
            unit = 'count'
        elif any(s in field for s in ('size', 'footprint', 'bytes', 'logical_writes')):
            unit = 'bytes'
        elif field in ('ri_user_time', 'ri_system_time', 'ri_runnable_time') or 'abstime' in field:
            unit = 'Mach absolute-time ticks; raw, no conversion'
        else:
            unit = 'raw SDK uint64; no physical-unit conversion or assumption'
        units[field] = unit
    return {'sdk_path': str(SDK), 'sdk_sha256': hashlib.sha256(raw).hexdigest(),
            'declaration': declaration, 'fields': fields, 'units': units,
            'flavor': 4, 'size_bytes': 16 + 8 * len(fields),
            'uuid': '16 raw bytes; not substituted for sampled native custody',
            'fault_limit': 'v4 supplies pageins; no minor/major-fault fields',
            'clock_limit': 'raw rusage fields never compared with Python monotonic origins'}

def validate_counter(row, previous=None):
    require(row['status'] == 0 and row['errno'] == 0, 'counter acquisition failed')
    require(type(row['pid']) is int and row['pid'] > 0, 'counter PID invalid')
    require(set(METRICS) <= set(row['values']), 'counter fields missing')
    require(all(uint(x) for x in row['values'].values()), 'invalid counter uint64')
    require(all(type(x) is int and 0 <= x < 256 for x in row['uuid']) and len(row['uuid']) == 16,
            'invalid counter UUID')
    require(row['values']['ri_instructions'] > 0 and row['values']['ri_cycles'] > 0,
            'instruction/cycle counter unavailable')
    if previous:
        require(row['pid'] == previous['pid'] and row['uuid'] == previous['uuid']
                and row['values']['ri_proc_start_abstime'] == previous['values']['ri_proc_start_abstime'],
                'counter process identity changed')
        for field in METRICS:
            require(row['values'][field] >= previous['values'][field], 'nonmonotonic counter ' + field)

class Counters:
    def __init__(self):
        self.layout = sdk_layout()
        self.usage = type('Usage', (ctypes.Structure,), {'_fields_':
            [('ri_uuid', ctypes.c_ubyte * 16)] + [(f, ctypes.c_uint64) for f in self.layout['fields']]})
        require(ctypes.sizeof(self.usage) == self.layout['size_bytes'], 'ctypes SDK size mismatch')
        self.lib = ctypes.CDLL('/usr/lib/libproc.dylib', use_errno=True)
        self.lib.proc_pid_rusage.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.c_void_p]
        self.lib.proc_pid_rusage.restype = ctypes.c_int
        self.previous = None

    def sample(self, pid):
        usage = self.usage()
        ctypes.set_errno(0)
        status = self.lib.proc_pid_rusage(pid, 4, ctypes.byref(usage))
        row = {'pid': pid, 'status': status, 'errno': ctypes.get_errno(),
               'uuid': list(usage.ri_uuid), 'values': {f: getattr(usage, f) for f in self.layout['fields']}}
        # Caller persists this even if validation rejects it.
        return row

    def accept(self, row):
        validate_counter(row, self.previous)
        self.previous = row

class Pairing:
    """Per-thread LIFO with one shared physical return site per thread/address.

    Recursive invocations can share a return PC; CFA selects the current call.
    A return on another frame or a crossing interval is a hard rejection.
    """
    def __init__(self):
        self.stacks = {}
        self.completed = []
        self.next_id = 0

    def enter(self, label, sequence, tid, pc, cfa, caller):
        require(label in RETURN_FAMILIES, 'unknown return family')
        require(all(uint(x) and x > 0 for x in (tid, pc, cfa, caller['pc'], caller['cfa'])), 'missing/invalid unwound caller')
        require(caller['valid'] is True and caller['executable'] is True, 'unproved unwound caller')
        require(caller['cfa'] >= cfa and caller['module'] and caller['uuid'], 'invalid caller stack/module')
        stack = self.stacks.setdefault(tid, [])
        require(not any(c['entry_cfa'] == cfa for c in stack), 'ambiguous/reused active stack frame')
        call = {'id': self.next_id, 'label': label, 'entry_sequence': sequence,
                'tid': tid, 'entry_pc': pc, 'entry_cfa': cfa, 'caller': dict(caller)}
        self.next_id += 1
        stack.append(call)
        return call

    def leave(self, sequence, tid, pc, cfa, module, uuid):
        stack = self.stacks.get(tid, [])
        require(bool(stack), 'unmatched return thread')
        call = stack[-1]
        expected = call['caller']
        require((pc, cfa, module, uuid) == (expected['pc'], expected['cfa'], expected['module'], expected['uuid']),
                'missing/ambiguous/out-of-order return stack identity')
        require(sequence > call['entry_sequence'], 'nonforward return')
        stack.pop()
        completed = dict(call, return_sequence=sequence)
        self.completed.append(completed)
        return completed

    def finish(self):
        require(not any(self.stacks.values()), 'missing required return coverage')
        require(set(c['label'] for c in self.completed) == RETURN_FAMILIES, 'missing return family')
        return list(self.completed)
