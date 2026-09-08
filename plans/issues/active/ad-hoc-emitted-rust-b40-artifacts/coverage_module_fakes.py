"""Explicit SYNTHETIC SB API model; no real debugger objects or execution.

Tokens stand for invented object pointers. None preserves unobserved identity.
Module metadata can be authentic replay data independently of those tokens.
"""
import copy
from types import SimpleNamespace
from coverage_symbols import INVALID


class NativeObject:
    def __init__(self, token):
        self.token, self.valid = token, True

    def IsValid(self):
        return self.valid

    def __eq__(self, other):
        if self.token is None or getattr(other, 'token', None) is None:
            return None
        return type(self) is type(other) and self.token == other.token

    def __ne__(self, other):
        result = self == other
        return None if result is None else not result


class Section(NativeObject):
    def __init__(self, module, row, parent=None):
        super().__init__(('SYNTHETIC-section', module.token, row['name']))
        self.module, self.row, self.parent = module, copy.deepcopy(row), parent
        self.children = []

    def GetName(self):
        return self.row['name']

    def GetFileAddress(self):
        return self.row['file']

    def GetLoadAddress(self, target):
        return self.row['load']

    def GetByteSize(self):
        return self.row['bytes']

    def GetParent(self):
        return self.parent if self.parent is not None else INVALID_OBJECT

    def GetNumSubSections(self):
        return len(self.children)

    def GetSubSectionAtIndex(self, index):
        return self.children[index]

    def FindSubSection(self, name):
        return next((s for s in self.children if s.GetName() == name), INVALID_OBJECT)


class Address:
    def __init__(self, section, offset):
        self.section, self.offset = section, offset
        self.module = section.module
        self.valid = True
        self.symbol_override = None

    def IsValid(self):
        return self.valid

    def GetModule(self):
        return self.module

    def GetSection(self):
        return self.section

    def GetOffset(self):
        return self.offset

    def GetFileAddress(self):
        return self.section.GetFileAddress() + self.offset

    def GetLoadAddress(self, target):
        load = self.section.GetLoadAddress(target)
        return INVALID if load == INVALID else load + self.offset

    def GetSymbol(self):
        if self.symbol_override is not None:
            return self.symbol_override
        return next((s for s in self.module.symbols.values()
                     if s.GetStartAddress().GetFileAddress() == self.GetFileAddress()), INVALID_OBJECT)


class Symbol(NativeObject):
    def __init__(self, module, row):
        super().__init__(('SYNTHETIC-symbol', module.token, row['name']))
        self.row = copy.deepcopy(row)
        section = module.FindSection('__TEXT').FindSubSection('__text')
        self.start = Address(section, row['start']['section_offset'])
        self.end = Address(section, row['end']['section_offset'])

    def GetName(self):
        return self.row['name']

    def GetMangledName(self):
        return self.row['mangled']

    def GetDisplayName(self):
        return self.row['display']

    def GetType(self):
        return self.row['type']

    def GetStartAddress(self):
        return self.start

    def GetEndAddress(self):
        return self.end


class Module(NativeObject):
    def __init__(self, row, entry, token='SYNTHETIC-module-A'):
        super().__init__(token)
        self.row = copy.deepcopy(row)
        self.sections = [Section(self, r) for r in row['sections']]
        start = entry['cached_or_app_symbol']['start']
        root = self.FindSection('__TEXT')
        root.children = [Section(self, {'name': '__text', 'file': start['section_file'],
                          'load': row['header_load'] + start['section_file'] - row['header_file'],
                          'bytes': start['section_size']}, root)]
        self.symbols = {entry['exact_lookup']: Symbol(self, entry['cached_or_app_symbol'])}
        self.context_owner = self
        self.context_count = 1
        self.header_override = None
        self.file_overrides = {}

    def GetFileSpec(self):
        return self.row['path']

    def GetUUIDString(self):
        return self.row['uuid']

    def GetTriple(self):
        return self.row['triple']

    def GetObjectFileHeaderAddress(self):
        if self.header_override is not None:
            return self.header_override
        root = self.FindSection('__TEXT')
        return Address(root, self.row['header_file'] - root.GetFileAddress())

    def FindSection(self, name):
        return next((s for s in self.sections if s.GetName() == name), INVALID_OBJECT)

    def ResolveFileAddress(self, value):
        if value in self.file_overrides:
            return self.file_overrides[value]
        for section in [c for s in self.sections for c in s.children] + self.sections:
            if section.GetFileAddress() <= value < section.GetFileAddress() + section.GetByteSize():
                return Address(section, value - section.GetFileAddress())
        raise ValueError('unmapped synthetic file address')

    def FindSymbols(self, name, kind):
        symbol = self.symbols.get(name, INVALID_OBJECT)
        context = SimpleNamespace(GetModule=lambda: self.context_owner, GetSymbol=lambda: symbol)
        return SimpleNamespace(GetSize=lambda: self.context_count,
                               GetContextAtIndex=lambda i: context)


class Target:
    def __init__(self, modules, anchor):
        self.modules, self.anchor, self.overrides = modules, anchor, {}
        self.on_resolve = lambda: None

    def ResolveLoadAddress(self, load):
        self.on_resolve()
        if load in self.overrides:
            return self.overrides[load]
        for section in [c for s in self.anchor.sections for c in s.children] + self.anchor.sections:
            start = section.GetLoadAddress(self)
            if start <= load < start + section.GetByteSize():
                return Address(section, load - start)
        raise ValueError('unmapped synthetic load address')


class Process:
    def __init__(self):
        self.stop, self.state, self.executable, self.region_success = 7, 5, True, True

    def IsValid(self):
        return True

    def GetStopID(self):
        return self.stop

    def GetState(self):
        return self.state

    def GetMemoryRegionInfo(self, load, region):
        region.start, region.end, region.executable = load, load + 4096, self.executable
        return SimpleNamespace(Success=lambda: self.region_success)


class Region:
    def GetRegionBase(self):
        return self.start

    def GetRegionEnd(self):
        return self.end

    def IsExecutable(self):
        return self.executable


INVALID_OBJECT = NativeObject('SYNTHETIC-invalid')
INVALID_OBJECT.valid = False
API = SimpleNamespace(eStateStopped=5, eSymbolTypeCode=2,
                      SBAddress=Address, SBMemoryRegionInfo=Region)
