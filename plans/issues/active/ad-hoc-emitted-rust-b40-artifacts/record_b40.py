"""Post-allocation records only: saved raw analysis and immutable-hash verification."""
import datetime
import hashlib
import json
import subprocess
from pathlib import Path

E=Path(__file__).resolve().parent
ROOT=E.parent/'sifr'
ART=ROOT/'plans/issues/active/ad-hoc-emitted-rust-b40-artifacts'

def digest(path):
    with Path(path).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()

def read(path):
    return json.loads(path.read_text()) if path.exists() else None

def save(path,value):
    path.write_text(json.dumps(value,indent=2,sort_keys=True)+'\n')

def main():
    frozen=read(E/'frozen-manifest.json')
    for path,h in frozen['files'].items():assert digest(path)==h,path
    allocation=read(E/'allocation-outcome.json')
    assert allocation is not None
    release=read(E/'complete-process-release.json')
    root=read(E/'launcher-exit.json')
    outcome=read(E/'boundaries/outcome.json') or {}
    source=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    owned_release=(release is not None and release.get('state')=='PASS'
                   and root is not None and root.get('root_exit') is not None)
    within=owned_release and allocation['elapsed_seconds']<=540
    target_records=[]
    for directory in sorted((E/'boundaries').glob('target-*')):
        observer=read(directory/'observer-result.json') or {}
        custody=read(directory/'custody.json') or {}
        first=(custody.get('rejections') or [{}])[0]
        n=first.get('observation',{}).get('native',{})
        target_records.append({'path':str(directory),'observer':observer,'handshake':read(directory/'inferior.json'),
                               'accepted_observations':len(custody.get('observations',[])),
                               'first_custody_reason':first.get('reason'),
                               'native_error':n.get('error'),'sequence':n.get('sequence'),
                               'acknowledged':(directory/'custody-ack.json').exists()})
    hashes={str(p):digest(p) for p in E.rglob('*') if p.is_file() and p.name!='terminal.json'}
    record={'item_id':'12K-B40','state':allocation['state']+' / TERMINAL STOP',
            'clone':str(ROOT),'branch':'codex/item12k-b40-urwpgv','base_sha':frozen['base_sha'],
            'source_sha':source,'PR':None,'merge_sha':None,'Opus':0,'Sifr_gates':0,
            'production_acceptance':False,'post_live_apparatus_edits':0,'post_live_tests':0,
            'allocation':allocation,'release':release,'actual_root_exit':root,
            'complete_release_within540':within,'resources_released':owned_release,
            'target_records':target_records,'suites':read(E/'offline-summary.json'),
            'frozen_paths_unchanged':len(frozen['files']),'evidence_sha256':hashes,
            'recorded_at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'remaining_scope':'Full B24 causation/unchanged10case representative/same-invocation budget/both selftests; B27 complete joint source/exact65/SQL/corpus/retained12/full phase OPEN',
            'next_action':'Preserve terminal, release/native retirement, parent/coordinator adjudication. This owner stops; no next item.'}
    save(E/'terminal-draft.json',record)
    save(ART/'diagnostic-outcome.json',{k:v for k,v in record.items() if k!='evidence_sha256'})
    checks={'state':'PASS','frozen_paths_unchanged':len(frozen['files']),
            'evidence_bytes':sum(p.stat().st_size for p in E.rglob('*') if p.is_file()),
            'complete_release_within540':within,'resources_released':owned_release}
    save(E/'terminal-record-checks.json',checks)
    for name in ('allocation-outcome.json','launcher-exit.json','complete-process-release.json',
                 'supervisor-inventory.json','analysis.json','analysis.md','terminal-record-checks.json'):
        if (E/name).exists():
            (ART/name).write_bytes((E/name).read_bytes())
    print(json.dumps({'state':record['state'],'resources_released':owned_release,
                     'within540':within,'targets':len(target_records),'checks':checks},indent=2))

if __name__=='__main__':main()
