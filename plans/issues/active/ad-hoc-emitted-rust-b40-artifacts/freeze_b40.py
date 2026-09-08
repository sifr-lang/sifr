"""Freeze exact diagnostic sources, passed named suites, and immutable inputs."""
import ast
import difflib
import hashlib
import json
import shutil
import subprocess
from pathlib import Path

E=Path(__file__).resolve().parent
ROOT=E.parent/'sifr'
OLD=Path('/private/tmp/sifr-b39.20ZQBg/evidence')
HOST=Path('/private/tmp/sifr-b39-host.8IkfrA/evidence')
ART=ROOT/'plans/issues/active/ad-hoc-emitted-rust-b40-artifacts'

def digest(path):
    with Path(path).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()

def save(path,value):
    path.write_text(json.dumps(value,indent=2,sort_keys=True)+'\n')

def nodes(path,excluded=()):
    tree=ast.parse(path.read_text().replace(str(E),str(OLD)).replace('12K-B40-urwpgv-target-','12K-B39-20ZQBg-target-'))
    tree.body=[n for n in tree.body if getattr(n,'name',None) not in excluded]
    return ast.dump(tree,include_attributes=False)

def main():
    assert not (E/'allocation.json').exists()
    assert not (E/'frozen-manifest.json').exists(), 'freeze already exists; preserve it before a correction'
    assert not (ROOT/'target').exists() and not (ROOT/'.git/objects/info/alternates').exists()
    base=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    assert base=='4b4cc339964baeeb6641e57dc669fef700a5fa24'
    counts={}
    for directory,field in ((HOST,'evidence_sha256'),(OLD,'evidence_sha256')):
        record=json.loads((directory/'terminal.json').read_text())
        for path,h in record[field].items():assert digest(path)==h,path
        counts[str(directory/'terminal.json')]=len(record[field])
        frozen=json.loads((directory/'frozen-manifest.json').read_text())
        for path,h in frozen['files'].items():
            expected=h['sha256'] if isinstance(h,dict) else h
            assert digest(path)==expected,path
        counts[str(directory/'frozen-manifest.json')]=len(frozen['files'])
    mapping=json.loads((E/'preservation-map.json').read_text())
    for value in mapping.values():assert digest(value['path'])==value['sha256']
    oldpre=json.loads((HOST/'prelaunch.json').read_text())
    stable={}
    for path,h in oldpre['file_hashes'].items():
        if not path.startswith('/private/tmp/sifr-'):
            assert digest(path)==h,path
            stable[path]=h
    for p in (ROOT/'.gitignore',ROOT/'sifr.toml'):
        assert digest(p)==digest(HOST.parent/'sifr'/p.relative_to(ROOT))
        stable[str(p)]=digest(p)
    workload=ROOT/'verification/areas/performance/formatter_project'
    assert len(list(workload.rglob('*.sifr')))==2
    for p in workload.rglob('*'):
        if p.is_file():
            assert digest(p)==digest(HOST.parent/'sifr'/p.relative_to(ROOT))
            stable[str(p)]=digest(p)
    for name,h in [('sifr-experiment09','a4386baecd7576b256dc6d68eb2e240952f591fa0a468df55f60080189c4b392'),
                   ('identity-contract.json','3ebc80789a4d9843badee45fb69587b12dd62101f0211a6b09d3a59896a1972d')]:
        assert digest(E/name)==h
        stable[str(E/name)]=h
    for name in ('counter-layout.json','counter-units.md'):
        assert digest(E/name)==digest(OLD/name)
    # Analyzer and every used data/function remain unchanged. Only observer
    # tests and coverage entrypoint self_test differ in their shared modules.
    for name,excluded in [('analyze_boundaries.py',()),('boundaries_core.py',()),
                         ('coverage_output.py',()),('coverage_symbols.py',('self_test',)),
                         ('boundaries_tests.py',('observer_tests',)),('boundaries_b39_tests.py',())]:
        assert nodes(E/name,excluded)==nodes(OLD/name,excluded), 'analyzer dependency changed: '+name
    analyzer=json.loads((OLD/'final-analyzer-receipt.json').read_text())
    assert analyzer['state']=='PASS' and analyzer['count']==84
    suites={}
    for kind,minimum in [('coverage',540),('observer',192)]:
        directory=sorted((E/'offline-b40').glob(kind+'-*'),key=lambda p:int(p.name.split('-')[-1]))[-1]
        receipt=json.loads((directory/'receipt.json').read_text())
        assert receipt['state']=='PASS' and receipt['count']>=minimum and receipt['exit']==0
        for name,h in receipt['source_hashes'].items():
            # Coverage does not execute boundary observer/inventory or record helpers.
            if kind=='coverage' and (name.startswith('boundaries_') or name in ('run_boundaries.py','prepare_b40.py','offline_b40.py','freeze_b40.py','record_b40.py')):continue
            assert digest(E/name)==h, 'passed suite source changed: '+kind+' '+name
        suites[kind]={'receipt':str(directory/'receipt.json'),'sha256':digest(directory/'receipt.json'),
                      'count':receipt['count'],'state':'PASS'}
    suites['analyzer']={'receipt':str(OLD/'final-analyzer-receipt.json'),'sha256':digest(OLD/'final-analyzer-receipt.json'),
                        'count':84,'state':'PASS','reused':True,'basis':'exact AST/function/data dependency comparison; analyzer byte-identical'}
    save(E/'offline-summary.json',suites)
    guards=subprocess.run(['python3','scripts/check_file_size_guardrails.py'],cwd=ROOT,capture_output=True,text=True)
    assert guards.returncode==0,guards.stdout+guards.stderr
    sizes={p.name:len(p.read_text().splitlines()) for p in E.glob('*.py')}
    assert max(sizes.values())<900,sizes
    ancestors=[]
    for original in oldpre['ancestor_config']:
        p=Path(original['path'].replace(str(HOST.parent),str(E.parent)))
        assert p.exists()==original['exists']
        if p.exists():assert digest(p)==original['sha256']
        ancestors.append(dict(original,path=str(p)))
    check={'state':'PASS','predecessor_counts':counts,'mapped_paths':len(mapping),
           'repo_file_size':guards.stdout,'external_source_lines':sizes,'analyzer_reused':suites['analyzer'],
           'binary':stable[str(E/'sifr-experiment09')],'contract':stable[str(E/'identity-contract.json')]}
    save(E/'identity-record-checks.json',check)
    changes=[]
    for p in sorted(E.glob('*.py'))+sorted(E.glob('*.lldb')):
        old=OLD/p.name
        before=old.read_text().replace(str(OLD),str(E)).replace('12K-B39-20ZQBg-target-','12K-B40-urwpgv-target-') if old.exists() else ''
        changes.extend(difflib.unified_diff(before.splitlines(True),p.read_text().splitlines(True),
                                           fromfile='original/'+p.name,tofile='B40/'+p.name))
    (E/'apparatus.diff').write_text(''.join(changes))
    files={str(p):digest(p) for p in sorted(E.rglob('*')) if p.is_file()}
    files.update(stable)
    amount=sum(p.stat().st_size for p in E.rglob('*') if p.is_file())
    assert amount<2*1024**3 and shutil.disk_usage(E).free>=4*1024**3
    manifest={'item':'12K-B40','base_sha':base,'branch':'codex/item12k-b40-urwpgv',
              'files':files,'evidence_bytes':amount,'source_hashes':{p.name:digest(p) for p in E.glob('*.py')},
              'suites':suites,'ancestor_config':ancestors,'immutable_predecessors':counts}
    save(E/'frozen-manifest.json',manifest)
    pre=dict(oldpre,item='12K-B40',base_sha=base,branch=manifest['branch'],
             manifest_sha256=digest(E/'frozen-manifest.json'),file_hashes={str(p):digest(p) for p in E.glob('*') if p.suffix in ('.py','.lldb')},
             ancestor_config=ancestors,debugger_command=['/usr/bin/lldb','--no-lldbinit','-b','-s',str(E/'boundaries.lldb')],
             launcher_command=['/opt/homebrew/bin/python3',str(E/'run_boundaries.py')],
             supervisor_command=['/opt/homebrew/bin/python3',str(E/'run_boundaries.py')],
             run_ids=['12K-B40-urwpgv-target-'+str(i) for i in range(6)],
             protocol='B39 shared lifecycle plus B40 initial group coherence and complete retirement-only obligations')
    pre.pop('readiness_sha256',None)
    pre['relocation_sha256']=digest(E/'relocation.json')
    pre['file_hashes'].update(stable)
    save(E/'prelaunch.json',pre)
    ART.mkdir()
    for p in sorted(E.glob('*.py'))+sorted(E.glob('*.lldb')):
        shutil.copyfile(p,ART/p.name)
    for name in ('frozen-manifest.json','prelaunch.json','offline-summary.json','identity-record-checks.json',
                 'relocation.json','preservation-map.json','apparatus.diff','preparation-failure.md'):
        shutil.copyfile(E/name,ART/name)
    shutil.copytree(E/'offline-b40',ART/'offline-b40')
    for p in E.glob('*.py'):p.chmod(0o444)
    for p in E.glob('*.lldb'):p.chmod(0o444)
    print(json.dumps({'state':'FROZEN','paths':len(files),'bytes':amount,'manifest':digest(E/'frozen-manifest.json'),
                      'prelaunch':digest(E/'prelaunch.json'),'suites':suites},indent=2))

if __name__=='__main__':main()
