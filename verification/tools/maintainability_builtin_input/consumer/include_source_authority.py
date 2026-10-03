"""Caller-held complete originals; projections cannot replace authority."""
from dataclasses import dataclass
import json
import os
from pathlib import Path
import shutil
import sys
import weakref

_SEAL=object()
_REGISTERED=weakref.WeakValueDictionary()


def decode(data):
    def closed(pairs):
        result={}
        for key,value in pairs:
            if key in result:raise ValueError('duplicate diagnostic field: '+key)
            result[key]=value
        return result
    return json.loads(data,object_pairs_hook=closed)


@dataclass(frozen=True)
class Authority:
    source_membership:bytes
    compiler_semantics:bytes
    stage_observations:bytes
    inputs:bytes
    seal:object


def require(condition,reason,api):
    if not condition:raise api.Unsupported(reason)


def register(source,semantic,stage,inputs,api):
    return restore((api.encoded(source),api.encoded(semantic),api.encoded(stage)),inputs,api)


def restore(originals,inputs,api):
    require(len(originals)==3 and all(type(v) is bytes for v in originals),'missing complete independent original bytes',api)
    value=Authority(*originals,api.encoded(inputs),_SEAL)
    _REGISTERED[id(value)]=value
    return value


def authenticate(authority,receipt,api):
    require(isinstance(authority,Authority) and authority.seal is _SEAL and _REGISTERED.get(id(authority)) is authority,'replaced/unregistered original include-source authority',api)
    inputs=decode(authority.inputs)
    require(len(inputs['original_inventory'])==3,'incomplete independent original authority inventory',api)
    for original,binding in zip((authority.source_membership,authority.compiler_semantics,authority.stage_observations),inputs['original_inventory']):
        require(api.digest(original)==binding['sha256']==inputs['files'].get(binding['path']),'replaced independent original authority bytes',api)
    require(api.encoded(receipt['inputs'])==authority.inputs,'replaced original include-source input inventory',api)
    require(api.run(['git','rev-parse','HEAD'],cwd=inputs['root']).stdout.strip()==inputs['source_candidate'] and list(os.uname())==inputs['host'] and sys.version==inputs['python_runtime'],'original candidate/host/runtime drift',api)
    require({k:api.digest(v.encode()) for k,v in os.environ.items() if not k.startswith('SIFR_BUILTIN_')}==inputs['parent_environment'],'original top-level consumer environment drift',api)
    for name,sha in inputs['files'].items():
        path=Path(name)
        require(path.is_file() and api.digest(path.read_bytes())==sha,'original include-source input drift: '+name,api)
    for command,expected in inputs['executable_selections'].items():
        current=shutil.which(command)
        require(current is not None and {'path':current,'resolved':str(Path(current).resolve())}==expected,'original executable selection drift: '+command,api)
    for name,expected in inputs['optional_input_slots'].items():
        path=Path(name);actual=api.digest(path.read_bytes()) if path.is_file() else None
        require(actual==expected and (not path.exists() or path.is_file()),'original configuration presence/content drift: '+name,api)
    for name,inventory in inputs['directories'].items():
        root=Path(name)
        actual=sorted(str(p.relative_to(root)) for p in root.rglob('*') if p.is_file() and (not inventory['exclude_operational'] or not set(p.relative_to(root).parts).intersection({'.git','target','__pycache__'})))
        require(actual==inventory['members'],'original directory membership drift: '+name,api)
    for key,value in inputs['build_environment'].items():require(os.environ.get(key)==value,'original build-script environment drift: '+key,api)
    for name,sha in inputs['tool']['consumer_sources'].items():require(api.digest((api.ROOT/'scripts'/name).read_bytes())==sha,'consumer source drift',api)
    for name,sha in inputs['tool']['helper_sources'].items():require(api.digest((api.TOOL/name).read_bytes())==sha,'helper/schema source drift',api)
    for name,sha in inputs['tool']['helper_build']['executables_and_runtime'].items():require(api.digest(Path(name).read_bytes())==sha,'helper executable/runtime drift',api)
    resolver=inputs['resolver_root']
    require(api.run(['git','-C',resolver,'rev-parse','HEAD']).stdout.strip()==api.RA_COMMIT and api.run(['git','-C',resolver,'rev-parse','HEAD^{tree}']).stdout.strip()==api.RA_TREE,'official RA pin drift',api)
    api.run(['git','-C',resolver,'diff','--exit-code','HEAD'])
    return inputs


def read_originals(authority):
    return decode(authority.source_membership),decode(authority.compiler_semantics),decode(authority.stage_observations)
