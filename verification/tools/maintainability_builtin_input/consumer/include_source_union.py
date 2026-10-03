"""Atomic complete diagnostic publication from independent verified contexts."""
from dataclasses import dataclass
import json
from pathlib import Path
import unittest
import uuid
import weakref
import include_source_encoding as canonical
from include_source_authority import require,restore,authenticate,decode
from include_source import verify

_SEAL=object()
_CONTEXTS=weakref.WeakValueDictionary()
_FIXTURES=weakref.WeakValueDictionary()


@dataclass(frozen=True)
class Context:
    receipt:bytes
    proof_path:str
    proof_digest:str
    qualification:bytes
    seal:object


@dataclass(frozen=True)
class Fixtures:
    candidate:str
    cases:tuple
    log_path:str
    log_digest:str
    seal:object


def qualify(proof,receipt,authority,output,api):
    qualification=verify(proof,receipt,authority,api)
    p=Path(output)/'proof.json'
    require(p.is_file() and canonical.file_digest(p)==receipt['proof_digest'],'publication proof differs from verified original relation',api)
    value=Context(api.encoded(receipt),str(p),receipt['proof_digest'],api.encoded(qualification),_SEAL)
    _CONTEXTS[id(value)]=value
    return value


def run_fixtures(names,directory,api):
    require(len(names)==13 and len(set(names))==13,'incomplete original thirteen-case fixture selection',api)
    destination=Path(directory)/('original-thirteen-fixture-union-'+str(uuid.uuid4())+'.log')
    suite=unittest.TestLoader().loadTestsFromNames(names)
    with destination.open('w') as stream:result=unittest.TextTestRunner(stream=stream,verbosity=2,failfast=True).run(suite)
    require(result.testsRun==13 and result.wasSuccessful(),'original thirteen-case fixture union failed or incomplete',api)
    value=Fixtures(api.run(['git','rev-parse','HEAD']).stdout.strip(),tuple(names),str(destination),api.digest(destination.read_bytes()),_SEAL)
    _FIXTURES[id(value)]=value
    return value


def publish(contexts,fixtures,destination,main_root,repeat_root,fixture_roots,api):
    destination=Path(destination)
    require(not destination.exists(),'complete diagnostic publication already exists',api)
    require(isinstance(fixtures,Fixtures) and fixtures.seal is _SEAL and _FIXTURES.get(id(fixtures)) is fixtures,'unregistered/replaced original fixture qualification',api)
    candidate=api.run(['git','rev-parse','HEAD'],cwd=main_root).stdout.strip()
    require(fixtures.candidate==candidate and api.digest(Path(fixtures.log_path).read_bytes())==fixtures.log_digest,'original fixture qualification input drift',api)
    required={(str(main_root),p,t) for p in ('sifr_codegen','sifr_lowering') for t in (False,True)}
    required|={(str(repeat_root),p,t) for p in ('sifr_codegen','sifr_lowering') for t in (False,True)}
    for root in fixture_roots:
        metadata=decode(api.run(['cargo','metadata','--locked','--no-deps','--format-version','1'],cwd=root).stdout)
        require(len(metadata['packages'])==1,'ambiguous actual fixture package',api)
        required.add((str(root),metadata['packages'][0]['name'],False))
    observed=set();records=[]
    for value in contexts:
        require(isinstance(value,Context) and value.seal is _SEAL and _CONTEXTS.get(id(value)) is value,'unregistered/replaced original context qualification',api)
        receipt=decode(value.receipt);inputs=receipt['inputs'];ctx=receipt['context']
        key=(inputs['root'],ctx['package'],ctx['test'])
        require(key not in observed and key in required,'duplicate/unexpected original context qualification',api);observed.add(key)
        original=tuple(Path(binding['path']).read_bytes() for binding in inputs['original_inventory'])
        authority=restore(original,inputs,api)
        # Every original file, cross-stage inventory and control input is
        # authenticated again before accepting the held consumer derivation.
        authenticate(authority,receipt,api)
        require(inputs['source_candidate']==candidate and ctx['target']=='x86_64-unknown-linux-gnu','original union candidate/target drift',api)
        require(canonical.file_digest(value.proof_path)==value.proof_digest==receipt['proof_digest'],'redigested/swapped publication projection',api)
        records.append({'context':ctx,'root':inputs['root'],'proof_path':value.proof_path,'proof_digest':value.proof_digest,'original_inventory':inputs['original_inventory'],'qualification':decode(value.qualification)})
        del original,authority
    require(observed==required,'missing required original context: no partial diagnostic publication',api)
    artifact={'schema':'sifr-maintainability-complete-include-source-diagnostic-v1','candidate':candidate,'semantic_export':False,'accepted_complete_union':True,'contexts':records,'original_fixture_cases':list(fixtures.cases),'fixture_log':fixtures.log_path,'fixture_log_digest':fixtures.log_digest}
    temporary=destination.with_suffix(destination.suffix+'.tmp');temporary.write_bytes(api.encoded(artifact));temporary.replace(destination)
    return artifact
