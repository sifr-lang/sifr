"""B33 stopped-snapshot native object identity; no launch/attach/continue API.

SBModule equality compares the underlying Module pointer, never UUID or path.
Every alias must prove that relation and agree with the current header mapping.
Receipt indices describe this observation only; they are not historical IDs.
"""
from coverage_symbols import INVALID, bind, expected_path, require, resolve_identity, uint


def equal(left, right):
    """Tri-state result: invalid/missing/incoherent equality is unknown."""
    try:
        if type(left) is not type(right) or not left.IsValid() or not right.IsValid():
            return None
        forward, reverse = left == right, right == left
        different, reversed_different = left != right, right != left
        if (type(forward) is not bool or type(reverse) is not bool
                or type(different) is not bool or type(reversed_different) is not bool
                or forward != reverse or different != reversed_different
                or forward == different):
            return None
        return forward
    except (AttributeError, TypeError, ValueError, RuntimeError):
        return None


def same(left, right, role):
    require(equal(left, right) is True, 'unknown or conflicting ' + role + ' instance')


def address_row(address, target):
    if not address.IsValid():
        return {'valid': False}
    section = address.GetSection()
    module = address.GetModule()
    header = module.GetObjectFileHeaderAddress()
    chain = []
    current = section
    while current.IsValid():
        require(len(chain) < 16, 'section parent cycle/depth')
        chain.append(current.GetName())
        current = current.GetParent()
    return {'valid': True, 'file': address.GetFileAddress(), 'load': address.GetLoadAddress(target),
            'section': list(reversed(chain)), 'section_offset': address.GetOffset(),
            'section_file': section.GetFileAddress(), 'section_size': section.GetByteSize(),
            'uuid': module.GetUUIDString(), 'header_file': header.GetFileAddress(),
            'header_relative': address.GetFileAddress() - header.GetFileAddress()}


def symbol_row(symbol, target):
    if not symbol.IsValid():
        return {'valid': False}
    return {'valid': True, 'name': symbol.GetName(), 'mangled': symbol.GetMangledName(),
            'display': symbol.GetDisplayName(), 'type': symbol.GetType(),
            'start': address_row(symbol.GetStartAddress(), target),
            'end': address_row(symbol.GetEndAddress(), target)}


def module_row(module, target):
    require(module.IsValid(), 'invalid module row')
    header = module.GetObjectFileHeaderAddress()
    return {'path': str(module.GetFileSpec()), 'uuid': module.GetUUIDString(),
            'triple': module.GetTriple(), 'header_file': header.GetFileAddress(),
            'header_load': header.GetLoadAddress(target)}


class Resolver:
    def __init__(self, api, target, process, entries, epoch, record):
        self.api, self.target, self.process = api, target, process
        self.entries, self.epoch, self.record = entries, epoch, record
        # Strong SBModule references retain identity across stops. Tokens are
        # assigned only from native equality; never from recorded scalar fields.
        self.instances = []
        self.receipts = []

    def stopped(self):
        require(self.process.IsValid() and self.process.GetState() == self.api.eStateStopped,
                'module snapshot requires stopped process')
        stop = self.process.GetStopID()
        require(type(stop) is int and stop > 0, 'invalid snapshot stop')
        return stop, self.epoch()

    def operation(self, label, action):
        stamp = self.stopped()
        receipt = {'label': label, 'stop_id': stamp[0], 'epoch': stamp[1],
                   'identity_basis': 'native SB equality at common stopped snapshot',
                   'state': 'REJECTED'}
        try:
            result = action(receipt)
            require(self.stopped() == stamp, 'stop/epoch changed during module snapshot')
            receipt['state'] = 'ACCEPTED'
            return result
        except BaseException as exc:
            receipt['error'] = str(exc)
            raise
        finally:
            self.receipts.append(receipt)
            self.record(self.receipts)

    def instance(self, module):
        comparisons = [equal(module, previous) for previous in self.instances]
        require(None not in comparisons, 'unknown retained module identity')
        matches = [index for index, match in enumerate(comparisons) if match]
        require(len(matches) <= 1, 'inconsistent retained module identity')
        if matches:
            return matches.pop()
        self.instances.append(module)
        return len(self.instances) - 1

    def section_tree(self, module):
        def walk(section, depth):
            require(depth < 16 and section.IsValid(), 'invalid section tree')
            same(self.api.SBAddress(section, 0).GetModule(), module, 'section owner')
            return {'name': section.GetName(), 'file': section.GetFileAddress(),
                    'load': section.GetLoadAddress(self.target), 'bytes': section.GetByteSize(),
                    'children': [walk(section.GetSubSectionAtIndex(i), depth + 1)
                                 for i in range(section.GetNumSubSections())]}
        return [walk(section, 0) for section in module.sections]

    def check_address(self, module, address, role, code=False):
        same(address.GetModule(), module, role + ' module')
        section = address.GetSection()
        same(self.api.SBAddress(section, 0).GetModule(), module, role + ' section owner')
        if code:
            text = module.FindSection('__TEXT')
            same(section.GetParent(), text, role + ' TEXT section')
            same(section, text.FindSubSection('__text'), role + ' text section')
        row = address_row(address, self.target)
        require(row['valid'] is True and uint(row['load']) and row['load'] > 0,
                role + ' unloaded address')
        require(uint(section.GetLoadAddress(self.target))
                and row['load'] == section.GetLoadAddress(self.target) + row['section_offset'],
                role + ' section load relation')
        back = self.target.ResolveLoadAddress(row['load'])
        same(back.GetModule(), module, role + ' roundtrip module')
        same(back.GetSection(), section, role + ' roundtrip section')
        require(address_row(back, self.target) == row, role + ' roundtrip address conflict')
        file_back = module.ResolveFileAddress(row['file'])
        same(file_back.GetModule(), module, role + ' file roundtrip module')
        same(file_back.GetSection(), section, role + ' file roundtrip section')
        require(address_row(file_back, self.target) == row, role + ' file roundtrip conflict')
        return row

    def same_sections(self, left, right):
        def compare(a, b):
            same(a, b, 'alias section')
            require(a.GetNumSubSections() == b.GetNumSubSections(), 'alias subsection count')
            for i in range(a.GetNumSubSections()):
                compare(a.GetSubSectionAtIndex(i), b.GetSubSectionAtIndex(i))
        require(len(left.sections) == len(right.sections), 'alias section count')
        for a, b in zip(left.sections, right.sections):
            compare(a, b)

    def select(self, label, required, receipt):
        entry = self.entries[label]
        path = expected_path(entry)
        listed = [(m, module_row(m, self.target)) for m in self.target.modules
                  if str(m.GetFileSpec()) == path]
        receipt['path_rows'] = [row for _, row in listed]
        mapped = [(m, row) for m, row in listed if row['header_load'] != INVALID]
        if not mapped:
            require(not required, ('unloaded required image', label))
            receipt['unloaded'] = True
            return None
        objects = [m for m, _ in mapped]
        matrix = [[equal(a, b) for b in objects] for a in objects]
        receipt['equality_matrix'] = matrix
        require(all(value is True for row in matrix for value in row),
                'distinct or unknown mapped module instances')
        # Only one proven instance remains. Resolve its uniquely agreed mapped
        # header to obtain the authoritative handle; no list-first selection.
        loads = {row['header_load'] for _, row in mapped}
        require(len(loads) == 1, 'alias header load conflict')
        load = loads.pop()
        require(uint(load) and load > 0, 'invalid mapped header')
        anchor_address = self.target.ResolveLoadAddress(load)
        anchor = anchor_address.GetModule()
        for module in objects:
            same(module, anchor, 'mapped header module')
        reference = module_row(anchor, self.target)
        require(reference['path'] == path and reference['uuid'] == entry['uuid']
                and reference['triple'] == entry['triple'], 'wrong mapped module identity')
        tree = self.section_tree(anchor)
        receipt['section_tree'] = tree
        for module, row in mapped:
            require(row == reference, 'conflicting alias module/header metadata')
            require(self.section_tree(module) == tree, 'conflicting alias section metadata')
            self.same_sections(module, anchor)
            header = module.GetObjectFileHeaderAddress()
            self.check_address(module, header, 'header')
            same(header.GetSection(), anchor_address.GetSection(), 'header section')
        # Check even unloaded aliases for contradictory representations of the
        # same object. Distinct unloaded modules are not current candidates.
        for module, row in listed:
            relation = equal(module, anchor)
            require(relation is not None, 'unknown listed module identity')
            require(not relation or row == reference, 'contradictory alias mapping')
        receipt['module'] = dict(reference, instance=self.instance(anchor))
        receipt['alias_count'] = len(objects)
        return anchor

    def current_module(self, label, required=True):
        return self.operation(label, lambda receipt: self.select(label, required, receipt))

    def resolve_symbol(self, label, receipt, supplied=None):
        module = self.select(label, True, receipt)
        if supplied is not None:
            same(supplied, module, 'supplied current module')
        # Every representation is independently checked, including its lookup.
        aliases = [m for m in self.target.modules if equal(m, module) is True]
        require(len(aliases) == receipt['alias_count'], 'module inventory changed within snapshot')
        symbols = []
        for alias in aliases + [module]:
            contexts = alias.FindSymbols(self.entries[label]['exact_lookup'], self.api.eSymbolTypeCode)
            require(contexts.GetSize() == 1, 'exact code context count is not one')
            context = contexts.GetContextAtIndex(0)
            same(context.GetModule(), module, 'symbol context module')
            symbol = context.GetSymbol()
            candidate = symbol_row(symbol, self.target)
            resolve_identity(self.entries[label], receipt['module'], [candidate])
            for endpoint in (symbol.GetStartAddress(), symbol.GetEndAddress()):
                self.check_address(alias, endpoint, 'symbol endpoint', code=True)
            symbols.append((symbol, candidate))
        canonical = symbols.pop()  # Explicit mapped-anchor lookup, not a match choice.
        for symbol, candidate in symbols:
            same(symbol, canonical[0], 'code symbol')
            require(candidate == canonical[1], 'conflicting alias symbol metadata')
        receipt['symbol'] = canonical[1]
        return module, canonical[0], canonical[1]

    def identity(self, label, supplied):
        def action(receipt):
            _, symbol, row = self.resolve_symbol(label, receipt, supplied)
            return receipt['module'], [row], symbol
        return self.operation(label, action)

    def initial_address(self, supplied, address):
        def action(receipt):
            module = self.select('initial_entry', True, receipt)
            same(supplied, module, 'initial supplied module')
            return self.check_address(module, address, 'initial PC', code=True)
        return self.operation('initial_entry', action)

    def resolve(self, label):
        def action(receipt):
            module, symbol, row = self.resolve_symbol(label, receipt)
            address = symbol.GetStartAddress()
            load = address.GetLoadAddress(self.target)
            back = self.target.ResolveLoadAddress(load)
            same(back.GetSymbol(), symbol, 'roundtrip symbol')
            region = self.api.SBMemoryRegionInfo()
            error = self.process.GetMemoryRegionInfo(load, region)
            live = {'load': load, 'section_load': address.GetSection().GetLoadAddress(self.target),
                    'roundtrip': address_row(back, self.target),
                    'roundtrip_path': str(back.GetModule().GetFileSpec()),
                    'region': {'success': error.Success(), 'executable': region.IsExecutable(),
                               'start': region.GetRegionBase(), 'end': region.GetRegionEnd()}}
            binding = bind(self.entries[label], receipt['module'], [row], live, self.epoch())
            receipt['binding'] = binding
            return binding
        return self.operation(label, action)
