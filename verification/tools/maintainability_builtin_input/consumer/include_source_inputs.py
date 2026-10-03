"""Complete original input census and independently owned normal artifacts."""
import copy
import json
import os
from pathlib import Path
import shutil
import source_binder
from include_source_authority import require


def census(root,metadata,messages,invocations,identity,output,target,api):
    inputs=source_binder._inputs(root,metadata,messages,invocations,identity,output,api)
    modules=root/'.gitmodules'
    if modules.is_file():inputs['files'][str(modules)]=api.digest(modules.read_bytes())
    links=[]
    for line in api.run(['git','ls-files','--stage'],cwd=root).stdout.splitlines():
        fields=line.split(None,3)
        if fields[0]!='160000':continue
        relative=fields[3];directory=root/relative
        initialized=(directory/'.git').exists()
        record={'path':relative,'expected':fields[1],'initialized':initialized}
        if initialized:
            record['actual']=api.run(['git','rev-parse','HEAD'],cwd=directory).stdout.strip()
            require(record['actual']==record['expected'],'original gitlink revision drift',api)
            api.run(['git','diff','--exit-code','HEAD'],cwd=directory)
        links.append(record)
    inputs['gitlinks']=links
    # Freeze every metadata package before semantic producers select any source.
    for package in metadata['packages']:
        directory=Path(package['manifest_path']).parent
        members=sorted(str(p.relative_to(directory)) for p in directory.rglob('*') if p.is_file() and not set(p.relative_to(directory).parts).intersection({'.git','target','__pycache__'}))
        inputs['directories'][str(directory)]={'members':members,'exclude_operational':True}
        for member in members:
            p=directory/member;inputs['files'][str(p)]=api.digest(p.read_bytes())
    sysroot=Path(api.run(['rustc','--print','sysroot']).stdout.strip())
    component_path=Path(os.environ['SIFR_BUILTIN_COMPONENT_RECEIPT'])
    component=json.loads(component_path.read_text())
    for relative,record in component['inventory'].items():
        p=sysroot/relative
        require(p.is_file() and api.digest(p.read_bytes())==record['sha256'],'official original component inventory drift',api)
        inputs['files'][str(p)]=record['sha256']
    for p in (component_path,component_path.parent/'official-channel-manifest.toml',component_path.parent/'rustc-dev-1.98.1-x86_64-unknown-linux-gnu.tar.xz',sysroot/'lib/rustlib/multirust-channel-manifest.toml',sysroot/'libexec/rust-analyzer-proc-macro-srv'):
        inputs['files'][str(p)]=api.digest(p.read_bytes())
    source_files={str(p):api.digest(p.read_bytes()) for p in (sysroot/'lib/rustlib/src/rust').rglob('*') if p.is_file()}
    inputs['files'].update(source_files)
    inputs['tool']=copy.deepcopy(inputs['tool']);inputs['tool']['rust_source_files']=source_files
    inputs['resolver_root']=str(next(Path.home().glob('.cargo/git/checkouts/rust-analyzer-*/03fcb77')))
    # Mutable target output paths are replaced only by exact context-owned copies.
    # The original paths, hashes and actual Cargo/argv relations remain recorded.
    snapshots=output/'normal-artifacts';snapshots.mkdir()
    bindings=[]
    for name,sha in list(inputs['files'].items()):
        p=Path(name)
        if p.is_relative_to(target):
            destination=snapshots/(api.digest(name.encode())+p.suffix)
            shutil.copyfile(p,destination)
            require(api.digest(destination.read_bytes())==sha,'normal artifact snapshot conflict',api)
            bindings.append({'original_path':name,'snapshot':str(destination),'sha256':sha})
            inputs['files'][str(destination)]=sha
    inputs['normal_artifact_bindings']=bindings
    return inputs


def unchanged(inputs,api):
    for name,sha in inputs['files'].items():
        p=Path(name);require(p.is_file() and api.digest(p.read_bytes())==sha,'original input drift across normal/compiler/native stages: '+name,api)


def finish(inputs,output,api):
    unchanged(inputs,api)
    # The normal artifacts were checked again after all producers, before the
    # immutable copies become authority independent of subsequent target reuse.
    for binding in inputs['normal_artifact_bindings']:
        require(inputs['files'].pop(binding['original_path'])==binding['sha256'],'original artifact binding conflict',api)
    for p in output.rglob('*'):
        if p.is_file() and p.name not in {'proof.json','receipt.json','success.json'}:
            inputs['files'][str(p)]=api.digest(p.read_bytes())
    return inputs
