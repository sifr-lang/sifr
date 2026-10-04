"""Explicit owned native cache continuation, preserving its failed observation."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import uuid

from sifr_verify.compressed_artifacts import compress,decoded_identity
from sifr_verify.execution_identity import artifact_identity
from sifr_verify.graph_retirement import GraphLease,active_builds
from metadata_artifact import require_native_binary
from published_predecessor import digest

RELATIVE='target/sysroot_release/cargo-target'


def inspect(root,previous,tools,target):
    previous=previous.absolute()
    if (previous.resolve(strict=True)!=previous or any(path.is_symlink() for path in (previous,*previous.parents))
            or previous.stat().st_uid!=os.getuid() or previous.stat().st_mode & 0o077):
        raise ValueError('continuation requires the private explicitly owned original preparation')
    state=previous/'state.json'
    if state.is_symlink(): raise ValueError('prior native state must be a regular owned file')
    old=json.loads(state.read_text())
    if (old.get('protocol')!='native-candidate-preparation-v1' or old.get('status')!='failed'
            or old.get('source_root')!=str(root) or old.get('target')!=target or old.get('tools')!=tools
            or old.get('cargo_lock_sha256')!=digest(root/'Cargo.lock') or old.get('runtime_assertions')!=0):
        raise ValueError('prior native preparation has unknown source/tool/target/dependency ownership')
    subprocess.run(['git','merge-base','--is-ancestor',old['source_commit'],'HEAD'],cwd=root,check=True,
                   stdout=subprocess.DEVNULL,stderr=subprocess.PIPE,timeout=30)
    producers=old['producer_sha256']
    if set(producers) not in ({'native_candidate.py','native_capacity.py'},
                             {'native_candidate.py','native_capacity.py','native_recovery.py'}):
        raise ValueError('prior native producer inventory is unknown')
    for name,sha in producers.items():
        raw=subprocess.check_output(['git','show',old['source_commit']+':verification/areas/sysroot_release/'+name],cwd=root,timeout=30)
        if hashlib.sha256(raw).hexdigest()!=sha: raise ValueError('prior native producer differs from recorded source')
    from native_candidate import commands
    for command in old['commands']:
        if (command['argv']!=commands(root,previous,old['version'],target).get(command['id'])
                or set(command['output'])!={'stdout','stderr'}):
            raise ValueError('prior native command inventory is unknown')
        for stream,row in command['output'].items():
            path=previous/(command['id']+'.'+stream)
            if row['path']!=str(path) or digest(path)!=row['sha256']: raise ValueError('prior native raw output differs')
    events=previous/'cargo.jsonl'
    if events.is_symlink(): raise ValueError('prior native events must be regular owned bytes')
    rows=[json.loads(line) for line in events.read_text().splitlines() if line.startswith('{')]
    artifacts=[row for row in rows if row.get('reason')=='compiler-artifact' and row.get('target',{}).get('name')=='sifr' and row.get('executable')]
    graph=previous/RELATIVE;binary=graph/target/'release/sifr'
    if graph.is_symlink() or any(path.is_symlink() for path in graph.rglob('*')):
        raise ValueError('links prevent safe owned native cache continuation')
    if len(artifacts)!=1 or artifacts[0]['executable']!=str(binary):
        raise ValueError('prior native cache has no unique completed compiler')
    profile=artifacts[0]['profile']
    if profile['opt_level']!='3' or profile['test'] or profile['debug_assertions'] or profile['overflow_checks']:
        raise ValueError('prior native compiler configuration differs')
    if (artifacts[0]['manifest_path']!=str(root/'crates/sifr/Cargo.toml')
            or artifacts[0]['target']['kind']!=['bin'] or artifacts[0]['target']['crate_types']!=['bin']
            or artifacts[0]['target']['src_path']!=str(root/'crates/sifr/src/main.rs')):
        raise ValueError('prior native compiler source/target differs')
    marker=previous/'target/.validation-graph-leases/cargo-target.json';data=json.loads(marker.read_text())
    owner=data['owner']
    if str(uuid.UUID(owner))!=owner or active_builds(root): raise ValueError('prior native graph has an active or unknown consumer')
    info=graph.stat()
    if data!={'owner':owner,'worktree':str(previous),'graph':RELATIVE,'device':info.st_dev,'inode':info.st_ino,'uid':os.getuid()}:
        raise ValueError('prior native graph ownership differs')
    return {'root':str(previous),'owner':owner,'marker':data,'state_sha256':digest(state),
            'cargo_events_sha256':digest(events),'graph':str(graph),'binary':str(binary)}


def acquire(record):
    lease=GraphLease(Path(record['root']),RELATIVE,record['owner']).acquire({'native-bundle-continuation'})
    if not lease.eligible:
        lease.close();raise ValueError('continuation cannot mutate an unknown native graph')
    return lease


def preserve(record,output):
    binary=Path(record['binary']);require_native_binary(binary,False)
    record=dict(record)
    record['original_compiler']=compress(binary,output/'original-compiler.gz',max_encoded_bytes=128*1024**2)
    directory=os.open(output,os.O_RDONLY)
    try: os.fsync(directory)
    finally: os.close(directory)
    return record


def check(record,output,root,tools,target):
    previous=Path(record['root']);state=previous/'state.json'
    # inspect validates original producer/source/commands and ownership, without
    # assuming the mutable cache still contains the old compiler version.
    current=inspect(root,previous,tools,target)
    for field in ('root','owner','marker','state_sha256','cargo_events_sha256','graph','binary'):
        if current[field]!=record[field]: raise ValueError('original continuation custody changed')
    copy=record['original_compiler'];retained=Path(copy['retained']['path'])
    if retained!=output/'original-compiler.gz' or artifact_identity(retained)!=copy['retained']:
        raise ValueError('retained original compiler bytes differ')
    if (copy['original']['path']!=record['binary'] or copy['decoded_size_bytes']>256*1024**2
            or decoded_identity(retained,limit=copy['decoded_size_bytes'])!=(copy['decoded_sha256'],copy['decoded_size_bytes'])
            or copy['decoded_sha256']!=copy['original']['sha256'] or copy['decoded_size_bytes']!=copy['original']['size_bytes']):
        raise ValueError('original compiler retention is incomplete')
    return Path(record['graph'])
