"""B33 actual duplicate-row replay and explicitly synthetic SB API negatives."""
import copy
import hashlib
import json
from pathlib import Path
from types import SimpleNamespace
from coverage_module import Resolver, equal
from coverage_module_fakes import API, Address, Module, Process, Target
from coverage_symbols import INVALID, expected_path, load_contract, require

RAW = Path('/private/tmp/sifr-b40.urwpgv/evidence/inputs/b32-events.json')
RAW_HASH = '503a922565c558967651362e8a9f5cc28742707ba5b2d6004ca9860d53254c61'


def run_tests(results):
    data = RAW.read_bytes()
    require(hashlib.sha256(data).hexdigest() == RAW_HASH, 'B32 raw changed')
    event = next(e for e in json.loads(data) if e['stop_id'] == 7)
    rows = [r for r in event['modules'] if r['path'] == '/usr/lib/dyld']
    require(len(rows) == 2 and rows[0] == rows[1], 'B32 duplicate representation changed')
    entries = load_contract()

    def check(name, action, rejection=None):
        error, accepted, passed = None, False, False
        try:
            action()
            accepted, passed = True, rejection is None
        except ValueError as exc:
            error = str(exc)
            passed = rejection is not None and rejection in error
        except Exception as exc:
            error = 'unexpected fixture/API exception: ' + repr(exc)
        results.append({'name': 'module-' + name, 'expected_accept': rejection is None,
                        'accepted': accepted, 'pass': passed, 'error': error,
                        'identity_provenance': 'unobserved for historical replay; all positive tokens synthetic'})

    def context(label='notifier', unknown=False, count=2):
        entry = entries[label]
        if label == 'notifier':
            source = copy.deepcopy(rows)
        else:
            start = entry['cached_or_app_symbol']['start']
            row = {'path': expected_path(entry), 'uuid': entry['uuid'], 'triple': entry['triple'],
                   'header_file': start['header_file'], 'header_load': 0x300000000,
                   'sections': [{'name': '__TEXT', 'file': start['header_file'],
                                 'load': 0x300000000,
                                 'bytes': start['section_file'] - start['header_file'] + start['section_size']}]}
            source = [row, copy.deepcopy(row)]
        token = None if unknown else 'SYNTHETIC-module-A'
        modules = [Module(source[i % 2], entry, token) for i in range(count)]
        # A third SBModule wrapper reached through mapped-address resolution,
        # distinct from every enumerated Python wrapper, with invented equality.
        anchor = Module(source[0], entry, token)
        target, process = Target(modules, anchor), Process()
        saved, epoch = [], [1]
        resolver = Resolver(API, target, process, entries, lambda: epoch[0],
                            lambda r: saved.append(copy.deepcopy(r[-1])))
        return SimpleNamespace(modules=modules, anchor=anchor, target=target, process=process,
                               saved=saved, epoch=epoch, resolver=resolver, label=label)

    def resolve(c):
        return c.resolver.resolve(c.label)

    def original_unknown():
        c = context(unknown=True)
        try:
            resolve(c)
        finally:
            require(c.saved[-1]['equality_matrix'] == [[None, None], [None, None]],
                    'historical identity was invented')
            require(c.saved[-1]['state'] == 'REJECTED', 'historical run retroactively accepted')
    check('actual-B32-stop7-identical-rows-identity-UNKNOWN', original_unknown,
          'distinct or unknown mapped module instances')

    for count in (1, 2, 3):
        def aliases(count=count):
            c = context(count=count)
            binding = resolve(c)
            require(binding['load'] == event['threads'][0]['pc'], 'replayed notifier coordinate mismatch')
            require(c.saved[-1]['alias_count'] == count and c.saved[-1]['state'] == 'ACCEPTED',
                    'missing alias evidence')
            require(c.resolver.current_module(c.label) is c.anchor,
                    'resolver returned enumerated first wrapper instead of mapped anchor')
        check('synthetic-native-alias-count-' + str(count), aliases)

    for label in entries:
        check('all13-current-mapped-symbol-' + label, lambda label=label: resolve(context(label)))

    def alias_symbol(c):
        return c.modules[1].symbols[entries[c.label]['exact_lookup']]

    def foreign(c):
        other = copy.deepcopy(c.anchor)
        other.token = 'SYNTHETIC-distinct-module-B'
        return other

    def changed_roundtrip(c, role, field):
        address = (c.anchor.GetObjectFileHeaderAddress() if role == 'header'
                   else c.anchor.symbols[entries[c.label]['exact_lookup']].GetStartAddress())
        load = address.GetLoadAddress(c.target)
        bad = copy.copy(address)
        bad.section = copy.copy(address.section)
        if field == 'module':
            bad.module = foreign(c)
        elif field == 'section':
            bad.section.token = ('SYNTHETIC-different-section', role)
        elif field == 'offset':
            bad.offset += 4
        elif field == 'symbol':
            bad.symbol_override = copy.deepcopy(alias_symbol(c))
            bad.symbol_override.token = 'SYNTHETIC-different-symbol'
        c.target.overrides[load] = bad

    negatives = [
        ('distinct-metadata-identical-modules', lambda c: setattr(c.modules[1], 'token', 'SYNTHETIC-distinct'),
         'distinct or unknown mapped module instances'),
        ('one-unknown-module', lambda c: setattr(c.modules[1], 'token', None), 'distinct or unknown'),
        ('invalid-module', lambda c: setattr(c.modules[1], 'valid', False), 'invalid module row'),
        ('wrong-module-UUID', lambda c: c.modules[1].row.update(uuid='wrong'), 'conflicting alias module'),
        ('wrong-module-triple', lambda c: c.modules[1].row.update(triple='wrong'), 'conflicting alias module'),
        ('alias-hidden-under-wrong-path', lambda c: c.modules[1].row.update(path='/wrong'), 'module inventory changed'),
        ('alias-header-file', lambda c: c.modules[1].row.update(header_file=rows[0]['header_file'] + 4),
         'alias header load conflict'),
        ('alias-header-load', lambda c: c.modules[1].sections[0].row.update(load=rows[0]['header_load'] + 4),
         'alias header load conflict'),
        ('alias-unloaded', lambda c: c.modules[1].sections[0].row.update(load=INVALID), 'contradictory alias mapping'),
        ('null-header-load', lambda c: [m.sections[0].row.update(load=0) for m in c.modules], 'invalid mapped header'),
        ('wrong-section-owner', lambda c: setattr(c.modules[1].sections[1], 'module', foreign(c)), 'section owner instance'),
        ('wrong-section-instance', lambda c: setattr(c.modules[1].sections[1], 'token', 'SYNTHETIC-foreign-section'),
         'alias section instance'),
        ('wrong-section-size', lambda c: c.modules[1].sections[1].row.update(bytes=1), 'alias section metadata'),
        ('wrong-section-file', lambda c: c.modules[1].sections[1].row.update(file=1), 'alias section metadata'),
        ('wrong-section-load', lambda c: c.modules[1].sections[1].row.update(load=1), 'alias section metadata'),
        ('wrong-context-owner', lambda c: setattr(c.modules[1], 'context_owner', foreign(c)), 'symbol context module'),
        ('zero-contexts', lambda c: setattr(c.modules[1], 'context_count', 0), 'exact code context count'),
        ('duplicate-contexts', lambda c: setattr(c.modules[1], 'context_count', 2), 'exact code context count'),
        ('wrong-symbol-instance', lambda c: setattr(alias_symbol(c), 'token', 'SYNTHETIC-other-symbol'), 'code symbol instance'),
        ('unknown-symbol-instance', lambda c: setattr(alias_symbol(c), 'token', None), 'code symbol instance'),
        ('wrong-symbol-name', lambda c: alias_symbol(c).row.update(name='wrong'), 'wrong exact symbol identity'),
        ('wrong-symbol-type', lambda c: alias_symbol(c).row.update(type=5), 'not a code symbol'),
        ('wrong-symbol-start', lambda c: setattr(alias_symbol(c).start, 'offset', 8), 'unexpected entry/extent'),
        ('wrong-symbol-end', lambda c: setattr(alias_symbol(c).end, 'offset', 8), 'unexpected entry/extent'),
        ('wrong-symbol-start-owner', lambda c: setattr(alias_symbol(c).start, 'module', foreign(c)), 'symbol endpoint module'),
        ('wrong-symbol-end-owner', lambda c: setattr(alias_symbol(c).end, 'module', foreign(c)), 'symbol endpoint module'),
        ('wrong-symbol-section-instance', lambda c: setattr(alias_symbol(c).start.section, 'token', 'SYNTHETIC-other-text'),
         'alias section instance'),
        ('wrong-header-roundtrip-module', lambda c: changed_roundtrip(c, 'header', 'module'), 'mapped header module'),
        ('wrong-header-roundtrip-section', lambda c: changed_roundtrip(c, 'header', 'section'), 'header roundtrip section'),
        ('wrong-header-roundtrip-offset', lambda c: changed_roundtrip(c, 'header', 'offset'), 'header roundtrip address conflict'),
        ('wrong-symbol-roundtrip-module', lambda c: changed_roundtrip(c, 'symbol', 'module'), 'symbol endpoint roundtrip module'),
        ('wrong-symbol-roundtrip-section', lambda c: changed_roundtrip(c, 'symbol', 'section'), 'symbol endpoint roundtrip section'),
        ('wrong-symbol-roundtrip-offset', lambda c: changed_roundtrip(c, 'symbol', 'offset'), 'symbol endpoint roundtrip address conflict'),
        ('wrong-symbol-roundtrip-symbol', lambda c: changed_roundtrip(c, 'symbol', 'symbol'), 'roundtrip symbol instance'),
        ('not-stopped', lambda c: setattr(c.process, 'state', 6), 'requires stopped process'),
        ('stop-changes-during-resolution', lambda c: setattr(c.target, 'on_resolve',
             lambda: setattr(c.process, 'stop', c.process.stop + 1)), 'stop/epoch changed'),
        ('epoch-changes-during-resolution', lambda c: setattr(c.target, 'on_resolve',
             lambda: c.epoch.__setitem__(0, c.epoch[0] + 1)), 'stop/epoch changed'),
        ('nonexecutable-region', lambda c: setattr(c.process, 'executable', False), 'unproved executable region'),
        ('failed-region-query', lambda c: setattr(c.process, 'region_success', False), 'unproved executable region'),
    ]
    for name, mutate, reason in negatives:
        def action(mutate=mutate):
            c = context()
            mutate(c)
            resolve(c)
        check(name, action, reason)

    for role in ('header', 'start', 'end'):
        def file_conflict(role=role):
            c = context()
            symbol = alias_symbol(c)
            address = (c.modules[1].GetObjectFileHeaderAddress() if role == 'header'
                       else symbol.GetStartAddress() if role == 'start' else symbol.GetEndAddress())
            bad = copy.deepcopy(address)
            bad.module = foreign(c)
            c.modules[1].file_overrides[address.GetFileAddress()] = bad
            resolve(c)
        check('file-roundtrip-foreign-' + role, file_conflict, 'file roundtrip module')

    def absent(required):
        c = context(count=0)
        require(c.resolver.current_module(c.label, required) is None, 'optional absent image not None')
    check('optional-image-unloaded', lambda: absent(False))
    check('required-image-unloaded', lambda: absent(True), 'unloaded required image')

    def stable_token():
        c = context()
        first = resolve(c)['module']['instance']
        c.process.stop += 1
        c.target.modules.reverse()
        require(resolve(c)['module']['instance'] == first, 'same native instance lost across stops')
    check('stable-native-identity-next-stop-reversed-alias-order', stable_token)

    def replaced_instance():
        c = context()
        old = resolve(c)['module']
        newer = context()
        for m in newer.modules + [newer.anchor]:
            m.token = 'SYNTHETIC-replacement-B'
        # All sections/symbols still compare among this new synthetic group.
        c.target.modules, c.target.anchor = newer.modules, newer.anchor
        c.process.stop += 1
        new = resolve(c)['module']
        require(new['instance'] != old['instance'], 'same metadata concealed replacement')
        require(new != old, 'observer binding comparison failed to detect replacement')
    check('same-metadata-replacement-visible-in-binding', replaced_instance)

    for mismatch in (False, True):
        def supplied(mismatch=mismatch):
            c = context()
            module = foreign(c) if mismatch else c.modules[1]
            c.resolver.identity(c.label, module)
        check('observer-identity-supplied-' + str(mismatch), supplied,
              'supplied current module' if mismatch else None)

    for mismatch in (False, True):
        def initial(mismatch=mismatch):
            c = context('initial_entry')
            symbol = c.anchor.symbols[entries[c.label]['exact_lookup']]
            address = copy.deepcopy(symbol.GetStartAddress())
            if mismatch:
                address.module = foreign(c)
            c.resolver.initial_address(c.modules[1], address)
        check('initial-PC-native-module-' + str(mismatch), initial,
              'initial PC module instance' if mismatch else None)

    class Incoherent(Module):
        def __ne__(self, other):
            return True
    def incoherent():
        a = Incoherent(rows[0], entries['notifier'])
        require(equal(a, a) is None, 'incoherent native API accepted')
    check('incoherent-equality-is-unknown', incoherent)
