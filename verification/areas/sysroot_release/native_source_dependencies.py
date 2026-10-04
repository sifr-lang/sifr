"""Prepare locked source dependencies before the native offline build."""
from datetime import datetime,timezone
import json,os
from pathlib import Path
import native_candidate
from native_capacity import resources
from published_predecessor import digest
from sifr_verify.execution_evidence import write_evidence
from sifr_verify.process_execution import execute
from sifr_verify.resource_admission import admit

ROOT=Path(__file__).resolve().parents[3]
INPUTS=('Cargo.toml','Cargo.lock','.cargo/config.toml','rust-toolchain.toml')

def inputs(root):
    return {name:digest(root/name) for name in INPUTS}

def commands(cargo):
    return [[cargo,'fetch','--locked'],[cargo,'fetch','--locked','--offline']]

def check(path,*,_pending=False):
    path=Path(path).resolve(strict=True);output=path.parent;report=json.loads(path.read_text())
    if not _pending and (path.name!='receipt.json' or json.loads((output/'state.json').read_text())!=report):
        raise ValueError('source dependencies require their completed official receipt')
    source=Path(report['source_root'])
    if (report['protocol']!='native-source-dependencies-v1' or report['status']!='prepared'
            or report['kind']!='preparation-output' or type(report['runtime_assertions']) is not int
            or report['runtime_assertions']!=0 or report['source_commit']!=native_candidate.clean_source(source)
            or report['producer_root']!=str(ROOT) or report['producer_commit']!=native_candidate.clean_source(ROOT)
            or report['producer_sha256']!=digest(Path(__file__)) or inputs(source)!=report['inputs_sha256']
            or report['tools']!=native_candidate.tool_identity()):
        raise ValueError('source dependency preparation identity differs')
    start,finish=(datetime.fromisoformat(report[name]) for name in ('started_utc','finished_utc'))
    now=datetime.now(timezone.utc)
    if start.utcoffset() is None or finish.utcoffset() is None or not start<=finish<=now or (now-finish).total_seconds()>86400:
        raise ValueError('source dependency preparation is stale or incomplete')
    if [row['argv'] for row in report['commands']]!=commands(report['tools']['cargo']['path']):
        raise ValueError('source dependency command inventory differs')
    for index,row in enumerate(report['commands']):
        if row['cause']!='exit' or type(row['returncode']) is not int or row['returncode']!=0 or row['truncated'] is not False or set(row['output_sha256'])!={'stdout','stderr'}:
            raise ValueError('source dependency preparation did not complete')
        for stream,sha in row['output_sha256'].items():
            if digest(output/(str(index)+'.'+stream))!=sha:
                raise ValueError('source dependency raw output differs')
    return report

def prepare(source,output):
    source=source.resolve(strict=True);output=output.absolute()
    if output.is_relative_to(source) or any(p.is_symlink() for p in (output,*output.parents)):
        raise ValueError('source dependency preparation requires private external output')
    output.mkdir(mode=0o700,parents=True,exist_ok=False)
    report={'protocol':'native-source-dependencies-v1','kind':'preparation-output','status':'incomplete',
        'runtime_assertions':0,'source_root':str(source),'source_commit':native_candidate.clean_source(source),
        'producer_root':str(ROOT),'producer_commit':native_candidate.clean_source(ROOT),
        'producer_sha256':digest(Path(__file__)),'inputs_sha256':inputs(source),
        'started_utc':datetime.now(timezone.utc).isoformat(),'commands':[]}
    def save(): (output/'state.json').write_text(json.dumps(report,indent=2)+'\n')
    save()
    try:
        report['tools']=native_candidate.tool_identity()
        cache=Path(os.environ.get('CARGO_HOME',str(Path.home()/'.cargo'))).resolve()
        cache.mkdir(parents=True,exist_ok=True)
        report['admission']=admit(resources(cache),dict(disk_growth_bytes=1024**3,retained_copy_bytes=0,
            disk_reserve_bytes=2*1024**3,memory_peak_bytes=512*1024**2,tmpfs_growth_bytes=0,memory_reserve_bytes=2*1024**3));save()
        env=os.environ.copy()|{'CARGO_NET_OFFLINE':'false','SIFR_VERIFY_DISK_FLOOR_PATH':str(cache),
                               'SIFR_VERIFY_DISK_FLOOR_BYTES':str(3*1024**3)}
        for index,argv in enumerate(commands(report['tools']['cargo']['path'])):
            result=execute(argv,cwd=source,env=env,deadline_seconds=900,limit_bytes=16*1024**2)
            raw={}
            for stream in ('stdout','stderr'):
                path=output/(str(index)+'.'+stream);path.write_bytes(getattr(result,stream));raw[stream]=digest(path)
            report['commands'].append(dict(argv=argv,cause=result.cause,returncode=result.returncode,
                truncated=result.truncated,output_sha256=raw));save()
            if result.cause!='exit' or result.returncode or result.truncated or inputs(source)!=report['inputs_sha256']:
                raise ValueError('locked source dependency preparation failed or changed inputs')
        report.update(status='prepared',finished_utc=datetime.now(timezone.utc).isoformat());save()
        check(output/'state.json',_pending=True);write_evidence(output/'receipt.json',report)
    except BaseException as error:
        report.update(status='failed',failure=type(error).__name__+': '+str(error),finished_utc=datetime.now(timezone.utc).isoformat());save();raise
    return report

if __name__=='__main__':
    import argparse
    parser=argparse.ArgumentParser(description=__doc__);subs=parser.add_subparsers(dest='command',required=True)
    build=subs.add_parser('prepare');build.add_argument('--source-root',type=Path,required=True);build.add_argument('--output',type=Path,required=True)
    verify=subs.add_parser('check');verify.add_argument('--receipt',type=Path,required=True)
    args=parser.parse_args()
    print(json.dumps(prepare(args.source_root,args.output) if args.command=='prepare' else check(args.receipt),indent=2))
