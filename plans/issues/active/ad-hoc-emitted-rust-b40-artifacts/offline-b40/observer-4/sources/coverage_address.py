"""B29 initial-entry coordinate guard; no LLDB import or live execution."""
import hashlib
import json
import sys
from pathlib import Path

DYLD_UUID = '74E52480-C2BD-3C8D-812D-95FE2B74A096'
ENTRY_OFFSET = 0x49c0
INVALID_ADDRESS = (1 << 64) - 1


def initial_entry_matches(row):
    """Validate one verified dyld image's header, TEXT, PC and executable map.

    LLDB file addresses need not use the disk image's zero-based coordinates.
    Subtract the corresponding image header in each domain, and independently
    verify the containing TEXT section's file-to-load mapping and bounds.
    """
    if (row['module_path'] != '/usr/lib/dyld'
            or row['pc_module_path'] != row['module_path']
            or row['module_uuid'] != DYLD_UUID
            or row['pc_module_uuid'] != DYLD_UUID
            or row['section_name'] != '__TEXT'
            or row['executable'] is not True):
        return False
    fields = ('pc', 'pc_file', 'header_file', 'header_load', 'section_file',
              'section_load', 'section_bytes', 'region_start', 'region_end')
    if any(type(row[key]) is not int or not 0 <= row[key] < INVALID_ADDRESS
           for key in fields):
        return False
    pc, pc_file = row['pc'], row['pc_file']
    header_file, header_load = row['header_file'], row['header_load']
    section_file, section_load = row['section_file'], row['section_load']
    size = row['section_bytes']
    if (size == 0 or section_file + size > INVALID_ADDRESS
            or section_load + size > INVALID_ADDRESS):
        return False
    return (
        pc - header_load == ENTRY_OFFSET
        and pc_file - header_file == ENTRY_OFFSET
        and section_file <= header_file <= pc_file < section_file + size
        and section_load <= header_load <= pc < section_load + size
        and pc_file - section_file == pc - section_load
        and header_file - section_file == header_load - section_load
        and row['region_start'] <= pc < row['region_end']
    )


def self_test():
    original = Path('/private/tmp/sifr-b28.prJZsY/evidence')
    inputs = {
        'terminal.json': '3722091a0bdfdea8e7070f13dcb3196028301b8a72c0cbecee9adba27daab614',
        'coverage/events.json': '9a19e58a30074723b12bdfddc35f463613d647b3ba7ce9e24940c63ce3508da2',
    }
    decoded = {}
    for name, expected in inputs.items():
        raw = (original / name).read_bytes()
        if hashlib.sha256(raw).hexdigest() != expected:
            raise ValueError('preserved B28 input changed: ' + name)
        decoded[name] = json.loads(raw)
    event = decoded['coverage/events.json'][0]
    module = next(m for m in event['modules'] if m['path'] == '/usr/lib/dyld')
    section = next(s for s in module['sections'] if s['name'] == '__TEXT')
    stop = decoded['terminal.json']['execution']['initial_stop']
    pc = event['threads'][0]['pc']
    positive = {
        'module_path': module['path'], 'module_uuid': module['uuid'],
        'pc_module_path': event['threads'][0]['module'],
        'pc_module_uuid': stop['uuid'], 'pc': pc,
        'pc_file': int(stop['pc_file'], 16),
        'header_file': module['header_file'], 'header_load': module['header_load'],
        'section_name': section['name'], 'section_file': section['file'],
        'section_load': section['load'], 'section_bytes': section['bytes'],
        # B28 stopped before saving a region: this is a synthetic executable
        # interval for offline predicate testing, never historical live evidence.
        'region_start': section['load'], 'region_end': section['load'] + section['bytes'],
        'executable': True,
    }
    cases = [('preserved-b28-coordinates-synthetic-executable-region', positive, True)]
    zero_file = dict(positive)
    for key in ('pc_file', 'header_file', 'section_file'):
        zero_file[key] -= positive['header_file']
    cases.append(('zero-based-file-domain', zero_file, True))
    relocated = dict(positive)
    for key in ('pc', 'header_load', 'section_load', 'region_start', 'region_end'):
        relocated[key] += 0x100000
    cases.append(('relocated-load-domain', relocated, True))
    changes = [
        ('wrong-entry-offset-both-domains', {'pc': pc + 4, 'pc_file': positive['pc_file'] + 4}),
        ('wrong-active-uuid', {'module_uuid': 'WRONG'}),
        ('wrong-pc-uuid', {'pc_module_uuid': 'WRONG'}),
        ('wrong-both-uuids', {'module_uuid': 'WRONG', 'pc_module_uuid': 'WRONG'}),
        ('wrong-active-path', {'module_path': '/other/dyld'}),
        ('wrong-pc-image', {'pc_module_path': '/other/dyld'}),
        ('wrong-file-mapping', {'pc_file': positive['pc_file'] + 4}),
        ('disk-offset-used-as-file-va', {'pc_file': ENTRY_OFFSET}),
        ('wrong-header-load', {'header_load': positive['header_load'] + 4}),
        ('wrong-header-file', {'header_file': positive['header_file'] + 4}),
        ('wrong-section-load', {'section_load': positive['section_load'] - 4}),
        ('wrong-section-file', {'section_file': positive['section_file'] - 4}),
        ('wrong-section-name', {'section_name': '__DATA'}),
        ('section-upper-bound-exclusive', {'section_bytes': ENTRY_OFFSET}),
        ('section-start-after-header', {'section_file': positive['section_file'] + 4,
                                        'section_load': positive['section_load'] + 4}),
        ('empty-section', {'section_bytes': 0}),
        ('overflowing-section', {'section_bytes': INVALID_ADDRESS - 1}),
        ('non-executable-region', {'executable': False}),
        ('region-upper-bound-exclusive', {'region_end': pc}),
        ('region-start-after-pc', {'region_start': pc + 1}),
        ('invalid-load-address', {'section_load': INVALID_ADDRESS}),
        ('invalid-file-address', {'pc_file': INVALID_ADDRESS}),
        ('negative-address', {'header_load': -1}),
    ]
    for name, delta in changes:
        cases.append((name, dict(positive, **delta), False))
    results = []
    for name, row, expected in cases:
        actual = initial_entry_matches(row)
        results.append({'name': name, 'expected': expected, 'actual': actual,
                        'pass': actual == expected})
    passed = all(case['pass'] for case in results)
    report = {
        'item': '12K-B29', 'state': 'PASS' if passed else 'FAIL',
        'source_inputs': {str(original / name): digest for name, digest in inputs.items()},
        'guard_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        'region_provenance': 'synthetic offline interval; B28 saved no live region',
        'preserved_case': positive, 'cases': results, 'count': len(results),
        'live_attempts': 0,
    }
    evidence = Path(__file__).resolve().parent / 'offline-result.json'
    with evidence.open('x') as stream:
        stream.write(json.dumps(report, indent=2, sort_keys=True) + '\n')
    print(json.dumps(report, indent=2, sort_keys=True), flush=True)
    return 0 if passed else 1


if __name__ == '__main__':
    if sys.argv[1:] != ['--self-test']:
        raise SystemExit('only --self-test is supported')
    raise SystemExit(self_test())
