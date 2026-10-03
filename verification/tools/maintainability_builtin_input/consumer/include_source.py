"""Diagnostic consumption/publication from independently held complete originals."""
import json
import weakref
from pathlib import Path
from include_source_authority import authenticate,read_originals,require,decode
from include_source_relation import project,SCHEMA,LOST,NOT_CLAIMED
import include_source_constraints
import include_source_dependency

_EXPECTED={}
RECEIPT='sifr-maintainability-include-source-receipt-v1'
TOP={'schema','semantic_export',*LOST,'source_native_attribute_membership','compiler_semantic_owner_identity','transformed_attribute_observations','input_binding'}


def _verify(proof,receipt,authority,api):
    inputs=authenticate(authority,receipt,api)
    schema=json.loads((api.TOOL/'schema/include-source-relation-v1.json').read_text())
    require(schema['additionalProperties'] is False and set(schema['required'])==TOP and set(schema['properties'])==TOP,'closed diagnostic schema authority mismatch',api)
    require(set(receipt)=={'schema','inputs','proof_digest','capture_status','semantic_export','context','timing_seconds'} and receipt['schema']==RECEIPT,'closed original include-source receipt mismatch',api)
    require(type(receipt['capture_status']) is int and receipt['capture_status']==0 and receipt['semantic_export'] is False,'failed original include-source capture',api)
    require(isinstance(proof,dict) and set(proof)==TOP and proof['schema']==SCHEMA and proof['semantic_export'] is False,'closed diagnostic include-source schema mismatch',api)
    require(all(proof[k]==NOT_CLAIMED for k in LOST),'unauthorized compiler-original attribute/lineage claim',api)
    source,semantic,stage=read_originals(authority)
    compiler=semantic['compiler'];native=source['native'];control=semantic['original_control'];invocation=semantic['selected_invocation'];context=receipt['context']
    command=['cargo','check','--locked','--tests' if context['test'] else '--lib','-p',context['package'],'--target','x86_64-unknown-linux-gnu','--message-format=json']
    require(control['command']==command and control['status']==0 and control['cwd']==inputs['root'],'wrong or failed original normal Cargo control',api)
    finished=[m for m in control['messages'] if m.get('reason')=='build-finished']
    require(len(finished)==1 and finished[0]['success'] is True,'missing successful original build-finished artifact authority',api)
    require(invocation['compiler_status']==0 and 'RUSTC_BOOTSTRAP' not in invocation['environment'],'failed/bootstrapped original selected invocation',api)
    args=invocation['args'];require(args[args.index('--crate-name')+1]==context['crate'] and ('--test' in args)==context['test'],'actual original invocation target/test mismatch',api)
    require(semantic['context']==context and type(context['test']) is bool and context['target']=='x86_64-unknown-linux-gnu','original receipt context conflict',api)
    invocations=semantic['complete_invocation_inventory']
    require(invocations.count(invocation)==1 and all(v['compiler_status']==0 and 'RUSTC_BOOTSTRAP' not in v['environment'] for v in invocations),'incomplete/ambiguous original invocation inventory',api)
    launcher=semantic['native_launcher'];expected_environment=control['environment'].copy()
    expected_environment.update(SIFR_BUILTIN_INCLUDE_DIAGNOSTIC='1',SIFR_BUILTIN_NATIVE_INVENTORY_DIR=launcher['environment']['SIFR_BUILTIN_NATIVE_INVENTORY_DIR'])
    if semantic['original_syn'] is not None:expected_environment['SIFR_BUILTIN_SOURCE_BINDER_RA']='1'
    require(launcher['status']==0 and launcher['cwd']==inputs['root'] and launcher['environment']==expected_environment,'native launcher did not use actual original top-level Cargo environment',api)
    artifacts={b['original_path']:b for b in inputs['normal_artifact_bindings']}
    required_artifacts={name for m in control['messages'] if m.get('reason')=='compiler-artifact' for name in m['filenames']}
    required_artifacts.update(a.split('=',1)[1] for v in invocations for a in v['args'] if '=' in a and a.split('=',1)[1].endswith(('.rmeta','.rlib','.so')))
    require(required_artifacts<=set(artifacts),'missing independently owned original normal artifact inventory',api)
    for binding in artifacts.values():require(inputs['files'].get(binding['snapshot'])==binding['sha256'],'original normal artifact byte binding conflict',api)
    require(stage['context']['crate']==compiler['crate'] and stage['context']['cfg']==compiler['cfg'] and stage['context']['target']==context['target'] and stage['context']['test']==context['test'],'transformed stage actual compiler context mismatch',api)
    require(compiler['schema']=='sifr-maintainability-source-binder-capture-v1' and compiler['stage']=='rustc-after-analysis' and compiler['semantic_export'] is False,'unknown original compiler semantic inventory',api)
    require(native['schema']=='development-public-include-token-inventory-v1' and native['accepted_proof'] is False and native['semantic_export'] is False,'unknown original native source inventory',api)
    intrinsic=[{'kind':'intrinsic-true','authority':'pinned-cfg::CfgOptions::default'}]
    require(native['intrinsic_cfg']==intrinsic and native['cfg'].count({'key':'true','value':None})==1 and [c for c in native['cfg'] if c!={'key':'true','value':None}]==compiler['cfg'],'original native/compiler cfg or explicit pinned intrinsic-true context mismatch',api)
    # Derivation is memoized only for this registered immutable authority. Every
    # consumption still authenticates all originals/inputs before comparing the
    # complete canonical relation; include roots remain separate actual contexts.
    key=id(authority)
    if key not in _EXPECTED or _EXPECTED[key][0]() is not authority:
        expected=project(source,semantic,stage,inputs,api)
        _EXPECTED[key]=(weakref.ref(authority),api.digest(api.encoded(expected)))
    actual=api.digest(api.encoded(proof))
    require(actual==_EXPECTED[key][1] and receipt['proof_digest']==actual,'projected include-source relation/cross-reference conflict with intact originals',api)
    return {'diagnostic_relation':True,'accepted_complete_union':False,'semantic_export':False,'owners':len(proof['compiler_semantic_owner_identity']['correspondences']),'include_contexts':len(proof['source_native_attribute_membership']['include_roots'])}


def derive(authority,receipt,api):
    inputs=authenticate(authority,receipt,api)
    expected=project(*read_originals(authority),inputs,api)
    _EXPECTED[id(authority)]=(weakref.ref(authority),api.digest(api.encoded(expected)))
    return expected


def verify(proof,receipt,authority,api):
    try:return _verify(proof,receipt,authority,api)
    except (KeyError,ValueError,TypeError,OSError,AttributeError,IndexError) as error:raise api.Unsupported('malformed/unavailable original include-source authority: '+str(error)) from error


def publish(proof,receipt,authority,path,api):
    result=verify(proof,receipt,authority,api)
    destination=Path(path);require(not destination.exists(),'diagnostic publication destination already exists',api)
    temporary=destination.with_suffix(destination.suffix+'.tmp')
    temporary.write_bytes(api.encoded({'proof':proof,'receipt':receipt,'qualification':result}))
    temporary.replace(destination)
    return result


def consume(data,receipt,authority,api):
    try:proof=decode(data)
    except (ValueError,TypeError) as error:raise api.Unsupported('malformed/duplicate diagnostic projection: '+str(error)) from error
    return verify(proof,receipt,authority,api)
