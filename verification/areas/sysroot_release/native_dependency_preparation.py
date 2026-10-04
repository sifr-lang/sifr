"""Online dependency preparation before offline published-runtime assertions."""
import json
import os
from pathlib import Path
import shutil
import tomllib
from native_capacity import resources
from published_predecessor import digest
from sifr_verify.process_execution import execute
from sifr_verify.resource_admission import admit

INPUTS=('Cargo.toml','Cargo.lock','crates/sifr_stdlib/Cargo.toml','crates/sifr_runtime/Cargo.toml')


def inputs(package):
    return {name:digest(package/name) for name in INPUTS}


def manifest(package):
    crate=package/'crates/sifr_stdlib'
    features=sorted(tomllib.loads((crate/'Cargo.toml').read_text())['features'])
    # Public stdlib features select their runtime leaves. Enabling every runtime
    # feature directly also selects compiler-private structural support, which
    # is deliberately absent from the published runtime package.
    return ('[package]\nname="published-runtime-preparation"\nversion="0.0.0"\nedition="2024"\n'
        '[workspace]\n[dependencies]\nsifr_stdlib = { path = '+json.dumps(str(crate))+
        ', default-features = false, features = '+json.dumps(features)+' }\n')


def commands(output):
    path=str(output/'Cargo.toml')
    return [['cargo','fetch','--manifest-path',path],
            ['cargo','fetch','--locked','--offline','--manifest-path',path]]


def prepare(package,output):
    output.mkdir(mode=0o700,exist_ok=False)
    before=inputs(package);(output/'src').mkdir();(output/'src/lib.rs').write_text('')
    (output/'Cargo.toml').write_text(manifest(package))
    # This is a new dependent project, not the published workspace. Cargo may
    # prune its seeded lock during preparation; the published lock stays intact.
    shutil.copyfile(package/'Cargo.lock',output/'Cargo.lock')
    cache=Path(os.environ.get('CARGO_HOME',str(Path.home()/'.cargo'))).resolve()
    cache.mkdir(parents=True,exist_ok=True)
    admission=admit(resources(cache),dict(disk_growth_bytes=512*1024**2,retained_copy_bytes=0,
        disk_reserve_bytes=2*1024**3,memory_peak_bytes=512*1024**2,tmpfs_growth_bytes=0,memory_reserve_bytes=2*1024**3))
    env=os.environ.copy()|{'CARGO_NET_OFFLINE':'false','SIFR_VERIFY_DISK_FLOOR_PATH':str(cache),
                           'SIFR_VERIFY_DISK_FLOOR_BYTES':str(3*1024**3)}
    report={'kind':'preparation-output','runtime_assertions':0,'status':'incomplete',
            'published_inputs_sha256':before,'commands':[],'admission':admission}
    def save(): (output/'state.json').write_text(json.dumps(report,indent=2)+'\n')
    save()
    try:
        for index,argv in enumerate(commands(output)):
            lock_before=digest(output/'Cargo.lock')
            result=execute(argv,cwd=output,env=env,deadline_seconds=600,limit_bytes=16*1024**2)
            raw={}
            for stream in ('stdout','stderr'):
                path=output/(str(index)+'.'+stream);path.write_bytes(getattr(result,stream));raw[stream]=digest(path)
            report['commands'].append(dict(argv=argv,cause=result.cause,returncode=result.returncode,
                truncated=result.truncated,output_sha256=raw));save()
            if result.cause!='exit' or result.returncode or result.truncated:
                raise ValueError('published runtime dependency preparation failed')
            if index and digest(output/'Cargo.lock')!=lock_before:
                raise ValueError('offline locked dependency validation changed prepared lock')
        if inputs(package)!=before:raise ValueError('dependency preparation changed published inputs')
        report.update(status='prepared',manifest_sha256=digest(output/'Cargo.toml'),lock_sha256=digest(output/'Cargo.lock'))
        save()
    except BaseException as error:
        report.update(status='failed',failure=type(error).__name__+': '+str(error));save();raise
    return report


def check(report,output,original_package,retained_package):
    if (json.loads((output/'state.json').read_text())!=report
            or report.get('kind')!='preparation-output' or type(report.get('runtime_assertions')) is not int
            or report.get('runtime_assertions')!=0
            or report.get('status')!='prepared' or inputs(retained_package)!=report['published_inputs_sha256']
            or digest(output/'Cargo.toml')!=report['manifest_sha256']
            or digest(output/'Cargo.lock')!=report['lock_sha256']
            or (output/'Cargo.toml').read_text()!=manifest_from_retained(original_package,retained_package)
            or [row['argv'] for row in report['commands']]!=commands(output)):
        raise ValueError('published dependency preparation identity differs')
    for index,row in enumerate(report['commands']):
        if row['cause']!='exit' or type(row['returncode']) is not int or row['returncode']!=0 or row['truncated'] is not False or set(row['output_sha256'])!={'stdout','stderr'}:
            raise ValueError('published dependency preparation is incomplete')
        for stream,sha in row['output_sha256'].items():
            if digest(output/(str(index)+'.'+stream))!=sha:raise ValueError('dependency preparation raw bytes differ')


def manifest_from_retained(original,retained):
    return manifest(retained).replace(json.dumps(str(retained/'crates/sifr_stdlib')),json.dumps(str(original/'crates/sifr_stdlib')))
