"""B32 exact identity, live binding and shared notifier transition contract."""
import copy
import hashlib
import json
import sys
from pathlib import Path

INVALID = (1 << 64) - 1
EARLY = frozenset(('prepare', 'fixups', 'generic'))
FAMILIES = EARLY | frozenset(('libsystem', 'sanitizers', 'constructor', 'main', 'completion'))
TRAPS = EARLY | frozenset(('notifier',))
E = Path('/private/tmp/sifr-b40.urwpgv/evidence')
BINARY = str(E / 'sifr-experiment09')
CONTRACT_HASH = '3ebc80789a4d9843badee45fb69587b12dd62101f0211a6b09d3a59896a1972d'


def require(condition, reason):
    if not condition:
        raise ValueError(reason)


def uint(value):
    return type(value) is int and 0 <= value < INVALID


def expected_path(entry):
    if entry['uuid'] == 'E527F68E-54F9-3463-ABC5-F8C8DD88A725':
        return BINARY
    return entry['module_path']


def resolve_identity(entry, module, candidates):
    """One typed, exact-key result; never choose among ambiguous candidates."""
    require(len(candidates) == 1, 'exact code context count is not one')
    symbol = candidates[0]
    expected = entry['cached_or_app_symbol']
    require(module['path'] == expected_path(entry), 'wrong module path')
    require(module['uuid'] == entry['uuid'] and module['triple'] == entry['triple'],
            'wrong module UUID or architecture')
    require(symbol['valid'] is True and symbol['type'] == 2, 'not a code symbol')
    require(all(symbol[k] == expected[k] for k in ('name', 'mangled', 'display')),
            'wrong exact symbol identity')
    domains = [expected]
    if 'disk_symbol' in entry:
        domains.append(entry['disk_symbol'])
    require(uint(module['header_file']) and any(module['header_file'] == s['start']['header_file']
                                              for s in domains), 'unknown file-address domain')
    for endpoint in ('start', 'end'):
        actual, wanted = symbol[endpoint], expected[endpoint]
        require(actual['valid'] is True and actual['uuid'] == entry['uuid'], 'invalid symbol address/UUID')
        require(actual['section'] == ['__TEXT', '__text'], 'wrong code section')
        for field in ('file', 'section_file', 'section_offset', 'section_size', 'header_file', 'header_relative'):
            require(uint(actual[field]), 'invalid address field ' + field)
        require(actual['header_file'] == module['header_file'], 'address/header domain mismatch')
        require(actual['file'] == actual['section_file'] + actual['section_offset'], 'wrong file/section relation')
        require(actual['file'] - actual['header_file'] == actual['header_relative'], 'wrong header relation')
        require(all(actual[k] == wanted[k] for k in ('section_offset', 'section_size', 'header_relative')),
                'unexpected entry/extent relative identity')
        require(actual['section_size'] > 0 and actual['section_file'] + actual['section_size'] < INVALID,
                'invalid section extent')
        require(actual['section_offset'] < actual['section_size'], 'address beyond code section')
    require(symbol['start']['file'] < symbol['end']['file'], 'empty or reversed symbol extent')
    return symbol


def bind(entry, module, candidates, live, epoch):
    symbol = resolve_identity(entry, module, candidates)
    start = symbol['start']
    require(type(epoch) is int and epoch >= 0, 'invalid epoch')
    require(uint(live['load']) and uint(live['section_load']) and uint(module['header_load']),
            'unloaded/invalid mapping')
    require(live['load'] > 0 and module['header_load'] > 0, 'null live mapping')
    require(live['load'] == live['section_load'] + start['section_offset'], 'wrong live section offset')
    require(live['load'] - module['header_load'] == start['header_relative'], 'wrong live header offset')
    require(live['section_load'] + start['section_size'] < INVALID, 'overflowed live section')
    back = live['roundtrip']
    require(back['valid'] is True and back['uuid'] == entry['uuid'], 'wrong roundtrip UUID')
    require(live['roundtrip_path'] == module['path'], 'wrong roundtrip module')
    require(all(back[k] == start[k] for k in ('section', 'section_offset', 'file', 'header_file')),
            'wrong roundtrip section/offset')
    require(back['load'] == live['load'], 'wrong roundtrip load')
    region = live['region']
    require(region['success'] is True and region['executable'] is True, 'unproved executable region')
    require(uint(region['start']) and uint(region['end']) and region['start'] <= live['load'] < region['end'],
            'entry outside executable region')
    return {'epoch': epoch, 'load': live['load'], 'module': dict(module),
            'symbol': copy.deepcopy(symbol), 'region': dict(region),
            'section_load': live['section_load']}


def check_site(binding, site, epoch):
    require(binding['epoch'] == epoch and site['epoch'] == epoch, 'stale site epoch')
    require(site['valid'] is True and site['enabled'] is True and site['resolved'] is True,
            'invalid/disabled/unresolved breakpoint site')
    require(site['locations'] == 1 and site['load'] == binding['load'], 'wrong location count/address')
    require(type(site['id']) is int and site['id'] > 0
            and type(site['location_id']) is int and site['location_id'] > 0, 'invalid site ID')


def breakpoint_pairs(data):
    """SBThread exposes uint64 words in exact breakpoint/location pairs."""
    require(isinstance(data, list) and len(data) >= 2 and len(data) % 2 == 0,
            'malformed breakpoint data length')
    require(all(type(x) is int and 0 < x <= INVALID for x in data[::2])
            and all(uint(x) and x > 0 for x in data[1::2]), 'malformed breakpoint ID')
    pairs = list(zip(data[::2], data[1::2]))
    require(len(set(pairs)) == len(pairs), 'duplicate breakpoint pair')
    return pairs


def stop_evidence(event):
    """Serialize one observed stop identically for live dispatch and raw replay."""
    require(len(event['threads']) == 1, 'ambiguous stopped threads')
    raw = event['threads'][0]
    is_bp = raw['reason'] == 3  # Installed eStopReasonBreakpoint; not description text.
    return {'breakpoint': is_bp, 'pc': raw['pc'], 'mode': raw['x0'],
            'count': raw['x1'], 'data': list(raw['data']),
            'pairs': breakpoint_pairs(raw['data']) if is_bp else [],
            'tid': raw['tid'], 'thread_count': len(event['threads']),
            'epoch': event['epoch'], 'stop_id': event['stop_id'],
            'module_count': len(event['modules']),
            'current_map': any(m['path'] == '/usr/lib/dyld'
                               and uint(m['header_load']) and m['header_load'] > 0
                               for m in event['modules'])}


class Protocol:
    """One explicit mode3->mode0 handover, exact sites and all eight families."""
    def __init__(self):
        self.epoch = 0
        self.phase = 'initial'
        self.handover_count = 0
        self.hits = []
        self.sites = {}
        self.last_stop = 0
        self.old_header = None
        self.prior_notifier = None

    def install(self, sites):
        require(TRAPS <= set(sites), 'missing early/notifier site')
        for value in sites.values():
            check_site(value['binding'], value['site'], self.epoch)
        self.sites = copy.deepcopy(sites)

    def initial(self, sites):
        require(self.phase == 'initial', 'repeated initial arming')
        self.install(sites)
        self.phase = 'active'

    def observe_stop(self, stop):
        require(type(stop) is int and stop > self.last_stop, 'reused/out-of-order stop')
        self.last_stop = stop

    def notifier_identity(self, event):
        """Single authority shared by dispatch and the state-changing protocol."""
        require(self.phase == 'active', 'unexpected notifier outside active epoch')
        saved = self.sites['notifier']
        check_site(saved['binding'], saved['site'], self.epoch)
        require(event['breakpoint'] is True, 'not a breakpoint stop')
        require(type(event['epoch']) is int and event['epoch'] == self.epoch
                and type(event['stop_id']) is int and event['stop_id'] == self.last_stop,
                'wrong active event epoch/stop')
        require(type(event['thread_count']) is int and event['thread_count'] == 1
                and uint(event['tid']) and event['tid'] > 0, 'invalid stopped thread identity')
        pairs = breakpoint_pairs(event['data'])
        require(pairs == event['pairs'], 'dispatch/protocol pair disagreement')
        own = (saved['site']['id'], saved['site']['location_id'])
        require(own in pairs, 'missing exact owned notifier pair')
        owned_ids = {v['site']['id'] for v in self.sites.values()}
        require([pair for pair in pairs if pair[0] in owned_ids] == [own],
                'competing or contradictory owned site')
        mode = event['mode']
        require(type(mode) is int and mode in (0, 1, 2, 3), 'unknown notifier mode')
        require(type(event['current_map']) is bool and type(event['module_count']) is int
                and event['module_count'] >= 0, 'invalid module map evidence')
        if event['pc'] != INVALID:
            require(uint(event['pc']) and event['pc'] == saved['binding']['load'],
                    'contradictory notifier PC')
        else:
            require(type(event['pc']) is int and mode == 3
                    and event['current_map'] is False and event['module_count'] == 0,
                    'invalid PC outside explicit empty-map mode3')
        require(event['current_map'] is True and event['module_count'] > 0
                or mode == 3 and event['current_map'] is False and event['module_count'] == 0,
                'unexpected empty or contradictory module map')
        if mode == 3:
            require(type(event['count']) is int and event['count'] == 1 and not self.hits and self.handover_count == 0,
                    'late/repeated/invalid handover')
            prior = self.prior_notifier
            require(prior is not None and prior['epoch'] == self.epoch
                    and prior['tid'] == event['tid'] and prior['stop_id'] < self.last_stop
                    and prior['saved'] == saved, 'no matching prior authenticated notifier epoch/thread/site')
            return 'handover'
        return 'library'

    def is_notifier(self, event):
        """Never bypass identity validation when dispatch sees a notifier hint."""
        require(self.phase == 'active', 'dispatch outside active epoch')
        saved = self.sites['notifier']
        pairs = breakpoint_pairs(event['data']) if event['breakpoint'] is True else []
        hinted = (any(pair[0] == saved['site']['id'] for pair in pairs)
                  or event['pc'] == saved['binding']['load']
                  or event['pc'] == INVALID or event['current_map'] is False)
        if hinted:
            self.notifier_identity(event)
        return hinted

    def notifier(self, event):
        kind = self.notifier_identity(event)
        saved = self.sites['notifier']
        if kind == 'handover':
            self.old_header = saved['binding']['module']['header_load']
            self.sites = {}
            self.prior_notifier = None
            self.phase = 'handover'
            self.epoch += 1
            return 'handover'
        self.prior_notifier = {'epoch': self.epoch, 'tid': event['tid'],
                               'stop_id': self.last_stop, 'saved': copy.deepcopy(saved)}
        return 'library'

    def handover_in(self, event, sites):
        require(self.phase == 'handover' and not self.hits, 'no pending handover')
        require(event['breakpoint'] is True and type(event['mode']) is int and event['mode'] == 0 and event['current_map'] is True,
                'handover did not return mapped mode0')
        self.install(sites)
        notifier = self.sites['notifier']['binding']
        require(event['pc'] == notifier['load'], 'wrong new notifier PC')
        require(notifier['module']['header_load'] != self.old_header, 'handover retained old mapping')
        self.handover_count += 1
        self.phase = 'active'

    def entry(self, label, event, value):
        require(self.phase == 'active' and self.handover_count == 1, 'entry without completed handover')
        require(label in FAMILIES, 'unknown required entry')
        check_site(value['binding'], value['site'], self.epoch)
        require(event['breakpoint'] is True and event['pc'] == value['binding']['load'], 'entry PC/reason mismatch')
        require((value['site']['id'], value['site']['location_id']) in event['pairs'], 'wrong hit location identity')
        if label in EARLY:
            require(label == 'prepare' or 'prepare' in self.hits, 'fixups before prepare')
        else:
            require(EARLY <= set(self.hits), 'later boundary before complete early coverage')
        self.hits.append(label)

    def finish(self, exit_code, file_phases):
        require(self.phase == 'active' and self.handover_count == 1, 'missing complete handover')
        require(set(self.hits) == FAMILIES, 'missing required entry coverage')
        require(exit_code == 0 and file_phases == 2, 'missing successful two-file completion')
        self.phase = 'complete'


def load_contract():
    raw = (E / 'identity-contract.json').read_bytes()
    require(hashlib.sha256(raw).hexdigest() == CONTRACT_HASH, 'static identity contract changed')
    return json.loads(raw)['entries']


def self_test():
    """One deterministic command, synthetic maps explicitly distinct from live evidence."""
    import ast
    from coverage_address import initial_entry_matches
    entries = load_contract()
    results = []

    def check(name, action, expected=True):
        try:
            action()
            actual = True
            error = None
        except (ValueError, KeyError, TypeError, IndexError) as exc:
            actual = False
            error = str(exc)
        results.append({'name': name, 'expected_accept': expected, 'accepted': actual,
                        'pass': actual == expected, 'error': error})

    def sample(label, disk=False, epoch=0):
        entry = entries[label]
        symbol = copy.deepcopy(entry['disk_symbol'] if disk else entry['cached_or_app_symbol'])
        start = symbol['start']
        header = 0x100000000 + epoch * 0x20000000
        module = {'path': expected_path(entry), 'uuid': entry['uuid'], 'triple': entry['triple'],
                  'header_file': start['header_file'], 'header_load': header}
        load = header + start['header_relative']
        back = dict(start, load=load)
        live = {'load': load, 'section_load': load - start['section_offset'],
                'roundtrip': back, 'roundtrip_path': module['path'],
                'region': {'success': True, 'executable': True, 'start': header,
                           'end': header + 0x10000000}}
        return entry, module, [symbol], live, epoch

    for label in entries:
        check('identity-and-binding-' + label, lambda label=label: bind(*sample(label)))
        if 'disk_symbol' in entries[label]:
            check('disk-and-relocated-' + label, lambda label=label: bind(*sample(label, True, 1)))

    mutations = [
        ('zero-contexts', lambda a: a[2].clear()),
        ('ambiguous-identical-contexts', lambda a: a[2].append(copy.deepcopy(a[2][0]))),
        ('wrong-name', lambda a: a[2][0].update(name='wrong')),
        ('wrong-mangled', lambda a: a[2][0].update(mangled='wrong')),
        ('wrong-display', lambda a: a[2][0].update(display='wrong')),
        ('wrong-type-trampoline', lambda a: a[2][0].update(type=5)),
        ('invalid-symbol', lambda a: a[2][0].update(valid=False)),
        ('wrong-module-UUID', lambda a: a[1].update(uuid='wrong')),
        ('wrong-path', lambda a: a[1].update(path='/other/dyld')),
        ('wrong-architecture', lambda a: a[1].update(triple='x86_64-apple-macosx')),
        ('wrong-address-UUID', lambda a: a[2][0]['start'].update(uuid='wrong')),
        ('wrong-code-section', lambda a: a[2][0]['start'].update(section=['__TEXT', '__auth_stubs'])),
        ('invalid-code-address', lambda a: a[2][0]['start'].update(valid=False)),
        ('empty-code-section', lambda a: a[2][0]['start'].update(section_size=0)),
        ('wrong-entry-offset', lambda a: a[2][0]['start'].update(section_offset=4)),
        ('wrong-header-offset', lambda a: a[2][0]['start'].update(header_relative=4)),
        ('wrong-file-domain', lambda a: a[1].update(header_file=1234)),
        ('reversed-extent', lambda a: a[2][0].update(end=copy.deepcopy(a[2][0]['start']))),
        ('unloaded-symbol', lambda a: a[3].update(load=INVALID)),
        ('unloaded-section', lambda a: a[3].update(section_load=INVALID)),
        ('unloaded-header', lambda a: a[1].update(header_load=INVALID)),
        ('null-live-address', lambda a: a[3].update(load=0)),
        ('wrong-load-offset', lambda a: a[3].update(load=a[3]['load']+4)),
        ('wrong-live-header', lambda a: a[1].update(header_load=a[1]['header_load']+4)),
        ('overflowed-section', lambda a: a[3].update(section_load=INVALID-2)),
        ('wrong-roundtrip-UUID', lambda a: a[3]['roundtrip'].update(uuid='wrong')),
        ('wrong-roundtrip-path', lambda a: a[3].update(roundtrip_path='/wrong')),
        ('wrong-roundtrip-offset', lambda a: a[3]['roundtrip'].update(section_offset=8)),
        ('wrong-roundtrip-address', lambda a: a[3]['roundtrip'].update(load=42)),
        ('nonexecutable-region', lambda a: a[3]['region'].update(executable=False)),
        ('region-query-failed', lambda a: a[3]['region'].update(success=False)),
        ('region-exclusive-upper-bound', lambda a: a[3]['region'].update(end=a[3]['load'])),
        ('region-start-after-entry', lambda a: a[3]['region'].update(start=a[3]['load']+4)),
        ('negative-address', lambda a: a[3].update(load=-1)),
        ('bool-address', lambda a: a[3].update(load=True)),
    ]
    for name, mutate in mutations:
        args = list(sample('prepare'))
        mutate(args)
        check(name, lambda args=args: bind(*args), False)
    cache = json.loads((E / 'module-inventory.json').read_text())
    for label, module_name, predicate in (
        ('sanitizer-real-trampoline', 'libSystem.B.dylib', lambda s: s['name'] == '_sanitizers_init'),
        ('jit-real-block-entry', 'dyld', lambda s: (s['mangled'] or '').endswith('EEEE_block_invoke')),
        ('generic-real-cold-entry', 'dyld', lambda s: 'applyFixupsGeneric' in s['name'] and '.cold.' in (s['mangled'] or '')),
    ):
        module = next(m for m in cache['modules'] if Path(m['path']).name == module_name)
        substitute = next(s for s in module['symbols'] if predicate(s))
        args = list(sample('sanitizers' if module_name == 'libSystem.B.dylib' else 'generic' if 'generic' in label else 'fixups'))
        args[2] = [substitute]
        check(label, lambda args=args: bind(*args), False)

    def sites(epoch):
        values = {}
        for index, label in enumerate(sorted(FAMILIES | TRAPS)):
            binding = bind(*sample(label, epoch=epoch))
            site = {'epoch': epoch, 'valid': True, 'enabled': True, 'resolved': True,
                    'locations': 1, 'load': binding['load'], 'id': index+1, 'location_id': 1}
            values[label] = {'binding': binding, 'site': site}
        return values

    def before_handover():
        protocol = Protocol()
        protocol.observe_stop(1)
        protocol.initial(sites(0))
        protocol.observe_stop(2)
        protocol.notifier(dict(outgoing(protocol), mode=0, current_map=True, module_count=2))
        protocol.observe_stop(3)
        return protocol

    def outgoing(protocol):
        site = protocol.sites['notifier']['site']
        data = [INVALID, 1, site['id'], site['location_id']]
        return {'breakpoint': True, 'pc': protocol.sites['notifier']['binding']['load'],
                'mode': 3, 'count': 1, 'current_map': False, 'module_count': 0,
                'data': data, 'pairs': breakpoint_pairs(data), 'tid': 123,
                'thread_count': 1, 'epoch': protocol.epoch, 'stop_id': protocol.last_stop}

    def after_handover():
        protocol = before_handover()
        protocol.observe_stop(4)
        protocol.notifier(outgoing(protocol))
        new = sites(1)
        protocol.observe_stop(5)
        protocol.handover_in({'breakpoint': True, 'mode': 0, 'current_map': True,
                              'pc': new['notifier']['binding']['load']}, new)
        return protocol

    def hit(protocol, label):
        value = protocol.sites[label]
        protocol.entry(label, {'breakpoint': True, 'pc': value['binding']['load'],
                              'pairs': [(value['site']['id'], value['site']['location_id'])]}, value)

    def full_sequence():
        p = after_handover()
        for label in ('prepare', 'fixups', 'generic', 'libsystem', 'sanitizers', 'constructor', 'main', 'completion'):
            hit(p, label)
        p.finish(0, 2)

    check('complete-epoch-handover-eight-family-sequence', full_sequence)
    for label in ('fixups', 'generic', 'libsystem', 'sanitizers', 'constructor', 'main', 'completion'):
        check('reject-out-of-order-' + label, lambda label=label: hit(after_handover(), label), False)
    check('missing-handover', lambda: hit(before_handover(), 'prepare'), False)
    check('missing-all-entries', lambda: after_handover().finish(0, 2), False)
    check('reused-stop-ID', lambda: before_handover().observe_stop(1), False)
    for key, bad in (('pc', 42), ('mode', 9), ('count', 0), ('breakpoint', False)):
        p = before_handover()
        event = dict(outgoing(p), **{key: bad})
        check('bad-handover-out-' + key, lambda p=p, event=event: p.notifier(event), False)
    p = before_handover()
    event = dict(outgoing(p), mode=0)
    check('unexpected-empty-map', lambda: p.notifier(event), False)
    for key, bad in (('mode', 3), ('pc', 42), ('current_map', False), ('breakpoint', False)):
        p = before_handover()
        p.notifier(outgoing(p))
        new = sites(1)
        event = {'breakpoint': True, 'mode': 0, 'current_map': True, 'pc': new['notifier']['binding']['load']}
        event[key] = bad
        check('bad-handover-in-' + key, lambda p=p, event=event, new=new: p.handover_in(event, new), False)
    p = after_handover()
    check('repeated-handover', lambda: p.notifier(outgoing(p)), False)
    p = before_handover()
    old = sites(0)
    p.notifier(outgoing(p))
    check('stale-bindings-after-handover', lambda: p.handover_in(
        {'breakpoint': True, 'mode': 0, 'current_map': True, 'pc': old['notifier']['binding']['load']}, old), False)
    p = before_handover()
    p.notifier(outgoing(p))
    old_load = sites(0)
    for value in old_load.values():
        value['binding']['epoch'] = 1
        value['site']['epoch'] = 1
    check('stale-addresses-with-new-epoch', lambda: p.handover_in(
        {'breakpoint': True, 'mode': 0, 'current_map': True, 'pc': old_load['notifier']['binding']['load']}, old_load), False)
    incomplete = sites(0)
    del incomplete['generic']
    check('missing-generic-site', lambda: Protocol().initial(incomplete), False)
    p = after_handover()
    hit(p, 'prepare')
    check('late-handover-after-entry', lambda: p.notifier(outgoing(p)), False)
    for key, bad in (('epoch', 0), ('valid', False), ('enabled', False), ('resolved', False), ('locations', 2), ('load', 42), ('location_id', 0)):
        value = copy.deepcopy(sites(1)['prepare'])
        value['site'][key] = bad
        check('bad-site-' + key, lambda value=value: check_site(value['binding'], value['site'], 1), False)
    p = after_handover()
    value = p.sites['prepare']
    check('wrong-hit-location-ID', lambda: p.entry('prepare', {'breakpoint': True,
        'pc': value['binding']['load'], 'pairs': [(value['site']['id'], 99)]}, value), False)
    check('wrong-hit-PC', lambda: p.entry('prepare', {'breakpoint': True, 'pc': 42,
        'pairs': [(value['site']['id'], value['site']['location_id'])]}, value), False)
    check('wrong-hit-reason', lambda: p.entry('prepare', {'breakpoint': False, 'pc': value['binding']['load'],
        'pairs': [(value['site']['id'], value['site']['location_id'])]}, value), False)
    for code, phases in ((1, 2), (0, 1), (0, 3)):
        p = after_handover()
        for label in ('prepare', 'fixups', 'generic', 'libsystem', 'sanitizers', 'constructor', 'main', 'completion'):
            hit(p, label)
        check('bad-completion-' + str((code, phases)), lambda p=p, code=code, phases=phases: p.finish(code, phases), False)
    raw = Path('/private/tmp/sifr-b40.urwpgv/evidence/inputs/b29-events.json').read_bytes()
    require(hashlib.sha256(raw).hexdigest() == 'af2f8db1ff5cea92f5a1af1e2fec4b20857466478e801f6a1db483081a22a7fa', 'B29 receipt changed')
    row = json.loads(raw)[0]['initial_coordinates']
    check('preserved-B29-actual-initial-coordinates', lambda: require(initial_entry_matches(row), 'initial mismatch'))
    helper = E / 'coverage_address.py'
    check('unchanged-B29-coordinate-helper', lambda: require(hashlib.sha256(helper.read_bytes()).hexdigest() ==
        '0df357c922d11386b5af80cfca7648d5542c8ebdec1072ba136a49be7c0bca9b', 'helper changed'))
    raw_b30 = Path('/private/tmp/sifr-b40.urwpgv/evidence/inputs/b30-events.json').read_bytes()
    require(hashlib.sha256(raw_b30).hexdigest() ==
            'a0c2ed89affe3c9d0e10498a4fa8d58f446f3d606dce1c74ebddf3c032fa8d3f', 'B30 raw changed')
    history = json.loads(raw_b30)
    previous = next(row for row in history if row['stop_id'] == 4)
    transition = next(row for row in history if row['stop_id'] == 6)

    def replay():
        p = Protocol()
        p.observe_stop(previous['stop_id'])
        p.initial(previous['sites_before_continue'])
        observed = stop_evidence(previous)
        require(p.is_notifier(observed), 'prior raw event missed by dispatch')
        require(p.notifier(observed) == 'library', 'prior raw event misclassified')
        p.observe_stop(transition['stop_id'])
        return p, stop_evidence(transition)

    def raw_handover():
        p, event = replay()
        require(p.is_notifier(event), 'raw mode3 missed by dispatch')
        require(p.notifier(event) == 'handover', 'raw mode3 rejected by protocol')
        require(p.epoch == 1 and p.phase == 'handover' and not p.sites
                and p.prior_notifier is None, 'old epoch not invalidated')

    check('preserved-B30-stop4-stop6-dispatch-and-protocol', raw_handover)

    def replace_data(p, event, data):
        event['data'] = data
        event['pairs'] = list(zip(data[::2], data[1::2]))

    cases = [
        ('missing-pair', lambda p, e: replace_data(p, e, [])),
        ('internal-only-pair', lambda p, e: replace_data(p, e, [INVALID, 1])),
        ('wrong-breakpoint', lambda p, e: replace_data(p, e, [INVALID, 1, 999, 1])),
        ('wrong-location', lambda p, e: replace_data(p, e, [INVALID, 1, 9, 2])),
        ('odd-words', lambda p, e: replace_data(p, e, [INVALID, 1, 9])),
        ('zero-location', lambda p, e: replace_data(p, e, [INVALID, 0, 9, 1])),
        ('negative-ID', lambda p, e: replace_data(p, e, [-1, 1, 9, 1])),
        ('bool-ID', lambda p, e: replace_data(p, e, [True, 1, 9, 1])),
        ('overflow-ID', lambda p, e: replace_data(p, e, [INVALID+1, 1, 9, 1])),
        ('duplicate-pair', lambda p, e: replace_data(p, e, [9, 1, 9, 1])),
        ('contradictory-owned-location', lambda p, e: replace_data(p, e, [9, 1, 9, 2])),
        ('competing-owned-entry', lambda p, e: replace_data(p, e, [9, 1, 6, 1])),
        ('competing-entry-wrong-location', lambda p, e: replace_data(p, e, [9, 1, 6, 2])),
        ('wrong-thread', lambda p, e: e.update(tid=e['tid']+1)),
        ('two-threads', lambda p, e: e.update(thread_count=2)),
        ('zero-thread', lambda p, e: e.update(tid=0)),
        ('wrong-event-epoch', lambda p, e: e.update(epoch=1)),
        ('wrong-stop', lambda p, e: e.update(stop_id=4)),
        ('wrong-site-epoch', lambda p, e: p.sites['notifier']['site'].update(epoch=1)),
        ('wrong-binding-epoch', lambda p, e: p.sites['notifier']['binding'].update(epoch=1)),
        ('wrong-prior-epoch', lambda p, e: p.prior_notifier.update(epoch=1)),
        ('wrong-prior-thread', lambda p, e: p.prior_notifier.update(tid=1)),
        ('wrong-prior-stop', lambda p, e: p.prior_notifier.update(stop_id=6)),
        ('missing-prior', lambda p, e: setattr(p, 'prior_notifier', None)),
        ('wrong-prior-binding', lambda p, e: p.prior_notifier['saved']['binding'].update(load=42)),
        ('wrong-mode0', lambda p, e: e.update(mode=0)),
        ('unknown-mode', lambda p, e: e.update(mode=4)),
        ('bool-mode', lambda p, e: e.update(mode=True)),
        ('wrong-count', lambda p, e: e.update(count=2)),
        ('bool-count', lambda p, e: e.update(count=True)),
        ('wrong-phase', lambda p, e: setattr(p, 'phase', 'handover')),
        ('prior-hit', lambda p, e: p.hits.append('prepare')),
        ('repeated-handover', lambda p, e: setattr(p, 'handover_count', 1)),
        ('contradictory-valid-PC', lambda p, e: e.update(pc=42)),
        ('invalid-PC-with-map', lambda p, e: e.update(current_map=True, module_count=1)),
        ('unmapped-but-nonempty', lambda p, e: e.update(module_count=1)),
        ('wrong-reason', lambda p, e: e.update(breakpoint=False)),
        ('pairs-disagree-with-raw', lambda p, e: e.update(pairs=[(9, 1)])),
    ]
    for name, mutate in cases:
        # Both paths must reject independently: dispatch cannot route an event
        # that the protocol rejects, and direct protocol use cannot skip guards.
        for route in ('dispatch', 'protocol'):
            p, event = replay()
            mutate(p, event)
            action = (lambda p=p, event=event: require(p.is_notifier(event), 'not dispatched')) if route == 'dispatch' else (
                lambda p=p, event=event: p.notifier(event))
            check('B30-replay-' + name + '-' + route, action, False)

    for mapped in (False, True):
        p, event = replay()
        event.update(pc=p.sites['notifier']['binding']['load'], current_map=mapped,
                     module_count=1 if mapped else 0)
        check('B30-valid-matching-PC-' + str(mapped),
              lambda p=p, event=event: require(p.is_notifier(event) and p.notifier(event) == 'handover', 'not accepted'))
    p, event = replay()
    replace_data(p, event, [9, 1])
    check('B30-owned-pair-without-internal', lambda: require(p.is_notifier(event)
          and p.notifier(event) == 'handover', 'owned pair alone rejected'))
    for bad in ([], [transition['threads'][0], transition['threads'][0]]):
        event = dict(transition, threads=bad)
        check('B30-raw-thread-count-' + str(len(bad)), lambda event=event: stop_evidence(event), False)
    p = after_handover()
    value = p.sites['prepare']
    event = dict(outgoing(p), pc=value['binding']['load'], current_map=True, module_count=2)
    replace_data(p, event, [value['site']['id'], value['site']['location_id']])
    check('normal-entry-dispatch-unchanged', lambda: require(not p.is_notifier(event), 'entry misrouted'))
    from coverage_output_tests import run_tests
    require(len(results) == 182, 'inherited identity/handover check count changed')
    run_tests(results)
    require(len(results) == 231, 'inherited B32 suite count changed')
    from coverage_module_tests import run_tests as module_tests
    module_tests(results)
    require(len(results) == 299, 'inherited B33 suite count changed')
    from coverage_custody_tests import run_tests as custody_tests
    custody_tests(results)
    require(len(results) == 349, 'inherited B34 suite count changed')
    from coverage_cleanup_tests import run_tests as cleanup_tests
    cleanup_tests(results)
    require(len(results) == 365, 'inherited B35 suite count changed')
    from coverage_lifecycle_tests import run_tests as lifecycle_tests
    lifecycle_tests(results)
    require(len(results) == 414, 'inherited B36 suite count changed')
    from coverage_b37_tests import run_tests as b37_tests
    b37_tests(results)
    require(len(results) == 443, 'inherited B37 suite count changed')
    from coverage_b38_tests import run_tests as b38_tests
    b38_tests(results)
    require(len(results) == 493, 'inherited B38 suite count changed')
    from coverage_b40_tests import run_tests as b40_tests
    b40_tests(results)
    for name in ('coverage_symbols.py', 'coverage_address.py', 'lldb_coverage.py', 'run_coverage.py',
                 'coverage_output.py', 'coverage_output_tests.py', 'coverage_module.py',
                 'coverage_module_fakes.py', 'coverage_module_tests.py',
                 'coverage_custody.py', 'coverage_custody_tests.py',
                 'coverage_cleanup.py', 'coverage_cleanup_tests.py', 'coverage_native.py',
                 'coverage_native_fakes.py', 'coverage_lifecycle_tests.py',
                 'coverage_acquisition.py', 'coverage_b37_tests.py', 'coverage_b38_tests.py',
                 'coverage_initial_group.py', 'coverage_b40_tests.py'):
        ast.parse((E / name).read_text(), feature_version=(3, 9))
    passed = all(r['pass'] for r in results)
    report = {'item': '12K-B40', 'state': 'PASS' if passed else 'FAIL', 'cases': results,
              'count': len(results), 'inherited_cases': 443, 'lifecycle_cases': len(results) - 443,
              'map_provenance': 'B29/B30 raw retained; B32 duplicate rows authentic, historical native identity UNKNOWN; positive module/section/symbol identities SYNTHETIC',
              'inferior_launches': 0, 'contract_sha256': CONTRACT_HASH,
              'source_hashes': {name: hashlib.sha256((E / name).read_bytes()).hexdigest()
                for name in ('coverage_symbols.py', 'coverage_address.py', 'lldb_coverage.py', 'run_coverage.py',
                             'coverage_output.py', 'coverage_output_tests.py', 'coverage.lldb',
                             'coverage_module.py', 'coverage_module_fakes.py', 'coverage_module_tests.py',
                             'coverage_custody.py', 'coverage_custody_tests.py',
                             'coverage_cleanup.py', 'coverage_cleanup_tests.py', 'coverage_native.py',
                             'coverage_native_fakes.py', 'coverage_lifecycle_tests.py',
                             'coverage_acquisition.py', 'coverage_b37_tests.py', 'coverage_b38_tests.py',
                             'coverage_initial_group.py', 'coverage_b40_tests.py')}}
    with (E / 'offline-result.json').open('x') as stream:
        json.dump(report, stream, indent=2, sort_keys=True)
        stream.write('\n')
    print(json.dumps({'state': report['state'], 'count': len(results),
                      'failures': [r for r in results if not r['pass']]}, indent=2))
    return 0 if passed else 1


if __name__ == '__main__':
    require(sys.argv[1:] == ['--self-test'], 'only --self-test supported')
    raise SystemExit(self_test())
