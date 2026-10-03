"""Fresh original normal controls, separate compiler replay and native capture."""
import gc
import json
import os
from pathlib import Path
import time
import source_binder
import include_source_inputs
from include_source_authority import register,restore,authenticate,decode,require
from include_source_relation import project
from include_source import verify,RECEIPT
import include_source_encoding as canonical

TARGET='x86_64-unknown-linux-gnu'
ORIGINALS=('source-original.json','semantic-original.json','stage-authority.json')


def context(package,test=False):
    return {'package':package,'crate':package.replace('-','_'),'test':test,'target':TARGET}


def load(output,identity,api):
    binding=decode((output/'success.json').read_bytes())
    for name,sha in binding.items():require(api.digest((output/name).read_bytes())==sha,'cached intact include-source original drift: '+name,api)
    receipt=decode((output/'receipt.json').read_bytes())
    expected={**identity,'rust_source_files':receipt['inputs']['tool']['rust_source_files']}
    require(receipt['inputs']['tool']==expected,'cached include-source helper/component identity drift',api)
    authority=restore(tuple((output/name).read_bytes() for name in ORIGINALS),receipt['inputs'],api)
    gc.collect()
    authenticate(authority,receipt,api)
    proof=decode((output/'proof.json').read_bytes());verify(proof,receipt,authority,api)
    print('include-source prepared cache HIT',receipt['context'],receipt['timing_seconds'],flush=True)
    return proof,receipt,authority


def capture(output,target,identity,api,ctx,root=None):
    output,target=Path(output).resolve(),Path(target).resolve();root=Path(root or api.ROOT).resolve()
    require(type(ctx['test']) is bool and ctx['target']==TARGET,'unsupported original target context',api)
    output.mkdir(parents=True,exist_ok=True)
    if (output/'success.json').is_file():return load(output,identity,api)
    started=time.monotonic();store=output/'invocations';store.mkdir()
    env=os.environ.copy()
    require('RUSTC_BOOTSTRAP' not in env and not env.get('RUSTC_WRAPPER') and not env.get('RUSTC_WORKSPACE_WRAPPER'),'unavailable original wrapper/bootstrap control authority',api)
    env.update(CARGO_INCREMENTAL='0',CARGO_BUILD_JOBS='2',CARGO_TARGET_DIR=str(target),RUSTC_WRAPPER=str(api.TOOL/'consumer/include_source_invocation.py'),SIFR_BUILTIN_ORIGINAL_INVOCATIONS=str(store))
    command=['cargo','check','--locked','--tests' if ctx['test'] else '--lib','-p',ctx['package'],'--target',TARGET,'--message-format=json']
    metadata_result=api.run(['cargo','metadata','--locked','--format-version','1','--filter-platform',TARGET],cwd=root,env=env,log=output/'metadata.log')
    metadata=decode(metadata_result.stdout)
    package=source_binder.unique([p for p in metadata['packages'] if p['name']==ctx['package']],'actual selected package',api)
    selected_target=source_binder.unique([t for t in package['targets'] if t['name']==ctx['crate'] and 'lib' in t['kind']],'actual selected library root',api)
    # Bytes are unchanged; request an actual selected Cargo invocation on a warm target.
    os.utime(selected_target['src_path'],None)
    normal=api.run(command,cwd=root,env=env,log=output/'normal-control.log')
    messages=[decode(line) for line in normal.stdout.splitlines() if line.startswith('{')]
    finished=[m for m in messages if m.get('reason')=='build-finished']
    require(len(finished)==1 and finished[0]['success'] is True,'missing original successful Cargo control',api)
    invocations=[decode(p.read_bytes()) for p in sorted(store.glob('*.json'))]
    def selected(values,name,is_test):
        return [v for v in values if '--crate-name' in v['args'] and v['args'][v['args'].index('--crate-name')+1]==name and ('--test' in v['args'])==is_test and '--target' in v['args'] and v['args'][v['args'].index('--target')+1]==TARGET]
    caller=source_binder.unique(selected(invocations,ctx['crate'],ctx['test']),'actual selected original invocation',api)
    syn_extern=[a.split('=',1)[1] for a in caller['args'] if a.startswith('syn=')]
    syn=None;rebuild=False
    if syn_extern:
        require(ctx['package']=='sifr_codegen','unqualified original syn caller',api)
        syn=source_binder.unique([p for p in metadata['packages'] if p['name']=='syn' and p['version']=='3.0.5'],'actual original syn package',api)
        matches=[v for v in selected(invocations,'syn',False) if v['environment'].get('CARGO_PKG_VERSION')=='3.0.5']
        if not matches:
            # Existing runner's bounded owned dependency rebuild obtains real argv.
            store.rename(output/'warm-control-invocations');store.mkdir()
            api.run(['cargo','clean','--package','syn@3.0.5','--target',TARGET],cwd=root,env=env,log=output/'owned-syn-rebuild.log')
            os.utime(selected_target['src_path'],None)
            normal=api.run(command,cwd=root,env=env,log=output/'normal-control-rebuild.log');rebuild=True
            messages=[decode(line) for line in normal.stdout.splitlines() if line.startswith('{')]
            invocations=[decode(p.read_bytes()) for p in sorted(store.glob('*.json'))]
            caller=source_binder.unique(selected(invocations,ctx['crate'],ctx['test']),'repeated actual selected original invocation',api)
            matches=[v for v in selected(invocations,'syn',False) if v['environment'].get('CARGO_PKG_VERSION')=='3.0.5']
        dependency=source_binder.unique(matches,'actual original dependency invocation',api)
    for v in invocations:require(v['compiler_status']==0 and 'RUSTC_BOOTSTRAP' not in v['environment'],'failed/bootstrapped original compiler inventory',api)
    control={'schema':'sifr-maintainability-original-build-v1','command':command,'cwd':str(root),'environment':env,'status':normal.returncode,'messages':messages,'metadata':metadata,'owned_dependency_rebuild':rebuild}
    (output/'original-build.json').write_bytes(api.encoded(control));(output/'caller-invocation.json').write_bytes(api.encoded(caller));(output/'original-invocation-inventory.json').write_bytes(api.encoded(invocations))
    before=include_source_inputs.census(root,metadata,messages,invocations,identity,output,target,api)
    before['selected_cargo_root']=selected_target['src_path']
    if syn is not None:
        archive=source_binder.unique(list(Path.home().glob('.cargo/registry/cache/*/syn-3.0.5.crate')),'original syn archive',api)
        control.update(syn_package=syn,archive=str(archive),archive_sha256=api.digest(archive.read_bytes()),archive_inventory=source_binder.archive_inventory(archive,syn,api))
        before['files'][str(archive)]=control['archive_sha256']
        (output/'original-build.json').write_bytes(api.encoded(control));before['files'][str(output/'original-build.json')]=api.digest((output/'original-build.json').read_bytes())
        (output/'dependency-invocation.json').write_bytes(api.encoded(dependency))
    (output/'inputs-before.json').write_bytes(api.encoded(before))
    replay=caller['environment'].copy();replay.update(SIFR_BUILTIN_SOURCE_BINDER=str(output/'compiler-original.json'),SIFR_BUILTIN_SOURCE_ENVELOPE=str(output/'stage-original.json'))
    if syn is not None:replay['SIFR_BUILTIN_SOURCE_BINDER_CALL_SUFFIX']='crates/sifr_codegen/src/inline_syntax.rs'
    analysis=api.run([str(target/'debug/sifr_maintainability_builtin_input'),*caller['args']],cwd=caller['cwd'],env=replay,log=output/'compiler-replay.log')
    raw=decode((output/'compiler-original.json').read_bytes());ra_input={'context':{'crate':ctx['crate']},'cfg':raw['cfg']}
    del raw;gc.collect()
    if syn is not None:
        replay=dependency['environment'].copy();replay['SIFR_BUILTIN_SOURCE_BINDER']=str(output/'syn-raw.json');replay['SIFR_BUILTIN_SOURCE_ENVELOPE']=str(output/'syn-stage-raw.json')
        dependency_analysis=api.run([str(target/'debug/sifr_maintainability_builtin_input'),*dependency['args']],cwd=dependency['cwd'],env=replay,log=output/'syn-replay.log')
        dependency_raw=decode((output/'syn-raw.json').read_bytes());(output/'independent-inventory.json').write_bytes(api.encoded(source_binder.inventory(dependency_raw)))
        ra_input.update(dependency_cfg=dependency_raw['cfg'],dependency_root=next(t['src_path'] for t in syn['targets'] if 'lib' in t['kind']),source_binder_call_suffix='crates/sifr_codegen/src/inline_syntax.rs');del dependency_raw;gc.collect()
    (output/'ra-input.json').write_bytes(api.encoded(ra_input))
    # Launch from the original top-level Cargo environment, never replay's caller env.
    raenv=env.copy();raenv.update(SIFR_BUILTIN_INCLUDE_DIAGNOSTIC='1',SIFR_BUILTIN_NATIVE_INVENTORY_DIR=str(output/'syntax-roots'))
    if syn is not None:raenv['SIFR_BUILTIN_SOURCE_BINDER_RA']='1'
    args=[str(target/'debug/ra_common'),str(root),ctx['package'],'included.rs',str(output/'ra-input.json')]
    if ctx['test']:args.append('test')
    native_result=api.run(args,cwd=root,env=raenv,log=output/'native.log');native=decode(native_result.stdout)
    (output/'native-original.json').write_bytes(api.encoded(native))
    raw=decode((output/'compiler-original.json').read_bytes());stage=decode((output/'stage-original.json').read_bytes())
    stage['dependency_inventory']=decode((output/'syn-stage-raw.json').read_bytes()) if syn is not None else None
    semantic={'compiler':raw,'original_control':control,'selected_invocation':caller,'complete_invocation_inventory':invocations,'context':ctx,'original_syn':None,'native_launcher':{'command':args,'cwd':str(root),'environment':raenv,'status':native_result.returncode}}
    if syn is not None:
        semantic['original_syn']={'original-build.json':control,'dependency-invocation.json':dependency,'caller-invocation.json':caller,'syn-raw.json':decode((output/'syn-raw.json').read_bytes()),'ra-source.json':native['original_syn_correspondence'],'independent-inventory.json':decode((output/'independent-inventory.json').read_bytes())}
    source={'native':native}
    canonical.write(output/'source-original.json',source);canonical.write(output/'semantic-original.json',semantic);canonical.write(output/'stage-authority.json',stage)
    inputs=include_source_inputs.finish(before,output,api)
    inputs['original_inventory']=[{'path':str(output/name),'sha256':api.digest((output/name).read_bytes())} for name in ORIGINALS]
    del source,semantic,stage,raw,native;gc.collect()
    authority=restore(tuple((output/name).read_bytes() for name in ORIGINALS),inputs,api)
    timing={'normal':normal.elapsed_seconds,'metadata':metadata_result.elapsed_seconds,'compiler':analysis.elapsed_seconds,'native':native_result.elapsed_seconds,'total_capture':time.monotonic()-started}
    if syn is not None:timing['syn_compiler']=dependency_analysis.elapsed_seconds
    receipt={'schema':RECEIPT,'inputs':inputs,'proof_digest':'','capture_status':0,'semantic_export':False,'context':ctx,'timing_seconds':timing}
    authenticate(authority,receipt,api)
    from include_source import derive
    proof=derive(authority,receipt,api);gc.collect()
    receipt['proof_digest']=canonical.digest(proof)
    # The producer derivation is still rechecked through the same public consumer.
    verify(proof,receipt,authority,api)
    canonical.write(output/'proof.json',proof);(output/'receipt.json').write_bytes(api.encoded(receipt))
    (output/'success.json').write_bytes(api.encoded({name:api.digest((output/name).read_bytes()) for name in (*ORIGINALS,'proof.json','receipt.json')}))
    print('include-source prepared cache MISS',ctx,timing,flush=True)
    return proof,receipt,authority
