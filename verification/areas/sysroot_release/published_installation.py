"""Exercise a published flat installer and independently check installed bytes."""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import sys
import tarfile
import tomllib

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT/'verification/runner'))
from sifr_verify.process_execution import execute
from sifr_verify.resource_admission import admit, discover
from published_predecessor import POLICY, digest, select
sys.path.insert(0, str(ROOT / "scripts/distribution"))
from qualify_stable_target import current_host_target

MEMBERS = ('.cargo', 'vendor', 'crates', 'lib', 'Cargo.toml', 'Cargo.lock', 'sysroot.toml', 'bin/sifr')


def installer_identity(policy, path):
    expected = policy['installer']
    name = 'sifr-installer-' + policy['version']
    url = f"https://github.com/sifr-lang/sifr/releases/download/{policy['version']}/{name}"
    if (expected['name'] != name or expected['browser_download_url'] != url
            or path.stat().st_size != expected['size'] or 'sha256:'+digest(path) != expected['digest']):
        raise ValueError('published installer differs from registered bytes')
    return {'path': str(path), 'sha256': digest(path), 'size_bytes': path.stat().st_size}


def payload(policy, archive, target, candidate_version):
    asset = select(policy, target, candidate_version)
    if archive.stat().st_size != asset['size'] or 'sha256:'+digest(archive) != asset['digest']:
        raise ValueError('published archive differs from registered bytes')
    rows, seen = [], set()
    with tarfile.open(archive, 'r:gz') as package:
        for member in package.getmembers():
            name = str(PurePosixPath(member.name))
            if (member.issym() or member.islnk() or not (member.isfile() or member.isdir())
                    or name.startswith('/') or '..' in PurePosixPath(name).parts or name in seen):
                raise ValueError('published installation requires a unique regular archive inventory')
            seen.add(name)
            if member.isfile():
                with package.extractfile(member) as stream:
                    hasher = hashlib.sha256()
                    for chunk in iter(lambda: stream.read(1024*1024), b''): hasher.update(chunk)
                rows.append({'path': name, 'sha256': hasher.hexdigest(), 'size_bytes': member.size})
        manifest = tomllib.loads(package.extractfile('sysroot.toml').read().decode())
    if (manifest['built-by-compiler-commit'] != policy['source_commit']
            or manifest['sifr-version'] != policy['version'] or manifest['target-triple'] != target):
        raise ValueError('published archive manifest differs from publication identity')
    return asset, rows


def verify_installed(root, rows):
    selected = [row for row in rows if any(row['path'] == member or row['path'].startswith(member+'/')
                                         for member in MEMBERS)]
    if not selected or not any(row['path'] == 'bin/sifr' for row in selected):
        raise ValueError('published installation inventory is incomplete')
    for row in selected:
        path = root/row['path']
        if (not path.is_file() or path.resolve() != path.absolute() or path.stat().st_size != row['size_bytes']
                or digest(path) != row['sha256']):
            raise ValueError('installed published payload differs: '+row['path'])
    actual = set()
    for member in MEMBERS:
        path = root/member
        entries = list(path.rglob('*')) if path.is_dir() else [path]
        if any(item.is_symlink() or not (item.is_file() or item.is_dir()) for item in entries):
            raise ValueError('installed published payload has links or special files')
        actual.update(str(item.relative_to(root)) for item in (path.rglob('*') if path.is_dir() else [path])
                      if item.is_file())
    if actual != {row['path'] for row in selected}:
        raise ValueError('installed published payload has missing or additional files')
    return len(selected)


def rehearse(policy, *, archive, installer, target, candidate_version, output):
    if target != current_host_target():
        raise ValueError('published installation requires its matching native host')
    for path in (archive, installer):
        if path.absolute() != path.resolve(strict=True) or not path.is_file():
            raise ValueError('published installation input must be a canonical regular file')
    archive, installer = archive.resolve(), installer.resolve()
    asset, rows = payload(policy, archive, target, candidate_version)
    installer_record = installer_identity(policy, installer)
    output = output.absolute()
    if output.is_relative_to(ROOT) or any(path.is_symlink() for path in (output, *output.parents)):
        raise ValueError('published installation rehearsal requires a private external output')
    output.parent.mkdir(parents=True, exist_ok=True)
    decoded = sum(row['size_bytes']+8192 for row in rows)
    resources = discover(disk_path=output.parent)
    # This is a bounded native installation observation, not the full cloud gate.
    admission = admit(resources, dict(disk_growth_bytes=decoded*2+asset['size'],retained_copy_bytes=0,
        disk_reserve_bytes=2*1024**3,memory_peak_bytes=1024**3,tmpfs_growth_bytes=0,memory_reserve_bytes=2*1024**3))
    output.mkdir(mode=0o700, exist_ok=False)
    report = {'schema_version':1,'protocol':'published-flat-installation-v1','status':'incomplete',
        'publication_source_commit':policy['source_commit'],'published_version':policy['version'],'target':target,
        'policy_sha256':hashlib.sha256(json.dumps(policy,sort_keys=True).encode()).hexdigest(),
        'producer_sha256':digest(Path(__file__)),'installer':installer_record,
        'archive':{'path':str(archive),'sha256':digest(archive)},'admission':admission,
        'started_utc':datetime.now(timezone.utc).isoformat(),'runtime_assertions':0,'commands':[]}
    def save():
        (output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n')
    save()
    env = os.environ.copy()
    for key in ('SIFR_SYSROOT','SIFR_SYSROOT_MODE','SIFR_INSTALL_MANIFEST_DIR','SIFR_SYSROOT_INSTALL_DIR',
                'SIFR_INSTALL_LOCK_HELD','SIFR_TEST_CHANNEL_METADATA_PATH','SIFR_CARGO','SIFR_RUSTC'):
        env.pop(key,None)
    transport=output/'transport';transport.mkdir();routes=transport/'routes.json'
    routes.write_text(json.dumps({asset['browser_download_url']:str(archive)}))
    curl=transport/'curl'
    curl.write_text('#!'+sys.executable+'\n'+'''import json,pathlib,shutil,sys
args=sys.argv[1:];routes=json.loads(pathlib.Path(__file__).with_name('routes.json').read_text())
urls=[arg for arg in args if arg.startswith(('https://','file://'))]
if len(urls)!=1 or urls[0] not in routes or '-o' not in args: raise SystemExit('unlisted installation URL')
shutil.copyfile(routes[urls[0]],args[args.index('-o')+1])
''');curl.chmod(0o700)
    managed=output/'managed'
    env.update(PATH=str(transport)+os.pathsep+env['PATH'],SIFR_INSTALL_DIR=str(managed/'bin'),
        SIFR_NO_MODIFY_PATH='1',SIFR_TARGET=target,
        SIFR_ARTIFACT_BASE_URL=asset['browser_download_url'].rsplit('/',1)[0],
        SIFR_VERIFY_DISK_FLOOR_BYTES=str(max(3*1024**3,int(env.get('SIFR_VERIFY_DISK_FLOOR_BYTES','0')))),
        SIFR_VERIFY_DISK_FLOOR_PATH=env.get('SIFR_VERIFY_DISK_FLOOR_PATH',str(output.parent)))
    def run(label, command):
        result=execute([str(arg) for arg in command],cwd=output,env=env,deadline_seconds=900,limit_bytes=16*1024**2)
        for stream in ('stdout','stderr'): (output/(label+'.'+stream)).write_bytes(getattr(result,stream))
        report['commands'].append({'id':label,'argv':[str(arg) for arg in command],
            'cause':result.cause,'returncode':result.returncode,'elapsed_seconds':result.elapsed_seconds,
            'truncated':result.truncated,
            'stdout_sha256':hashlib.sha256(result.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(result.stderr).hexdigest()})
        save()
        if result.cause!='exit' or result.returncode or result.truncated: raise ValueError(label+': incomplete or failed')
        return result.stdout
    try:
        for label, flags in (('install',[]),('reinstall',['--force'])):
            run(label,['sh',installer,'--no-modify-path',*flags])
            report['verified_installed_files']=verify_installed(managed,rows)
            if digest(installer)!=installer_record['sha256'] or digest(archive)!=report['archive']['sha256']:
                raise ValueError('published transport inputs changed')
        binary=managed/'bin/sifr'
        if run('version',[binary,'--version']).strip()!=f"sifr {policy['version']}".encode():
            raise ValueError('published installed version differs')
        identity=json.loads(run('sysroot',[binary,'--print','sysroot','--json']))
        doctor=json.loads(run('doctor',[binary,'doctor','--json']))
        if (identity['built_by_compiler_commit']!=policy['source_commit'] or identity['target_triple']!=target
                or Path(identity['root'])!=managed or doctor['status']!='ok'):
            raise ValueError('published installed compiler identity or doctor differs')
        report.update(status='legacy-installed',runtime_assertions=3,managed_root=str(managed),
                      supported_cli=['--version','--print sysroot --json','doctor --json'],
                      claim='native published install/reinstall and payload identity only; upgrade not yet qualified')
    except BaseException as error:
        report.update(status='failed',failure=type(error).__name__+': '+str(error));raise
    finally:
        report['finished_utc']=datetime.now(timezone.utc).isoformat();save()
    return report


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ('archive','installer','output'): parser.add_argument('--'+name,type=Path,required=True)
    parser.add_argument('--target',required=True);parser.add_argument('--candidate-version',required=True)
    args=parser.parse_args()
    print(json.dumps(rehearse(json.loads(POLICY.read_text()),**vars(args)),indent=2))


if __name__=='__main__': main()
