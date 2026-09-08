"""Preserve exact source and each named suite receipt, including failed attempts."""
import ast
import hashlib
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

E=Path(__file__).resolve().parent

def digest(path):
    with Path(path).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()

def main():
    name=sys.argv[1]
    assert name in ('coverage','observer')
    assert not (E/'allocation.json').exists(), 'no post-live tests'
    root=E/'offline-b40';root.mkdir(exist_ok=True)
    directory=root/(name+'-'+str(len(list(root.glob(name+'-*')))+1));directory.mkdir()
    sources=directory/'sources';sources.mkdir()
    hashes={}
    for path in sorted(E.glob('*.py'))+sorted(E.glob('*.lldb')):
        shutil.copyfile(path,sources/path.name);hashes[path.name]=digest(path)
    command=['/opt/homebrew/bin/python3',str(E/('coverage_symbols.py' if name=='coverage' else 'run_boundaries.py')),'--self-test']
    receipt={'command':command,'cwd':str(E.parent/'sifr'),'source_hashes':hashes,
             'TMPDIR':str(E.parent/'tmp'),'CARGO_TARGET_DIR':'unset','sources':str(sources)}
    (directory/'command.json').write_text(json.dumps(receipt,indent=2)+'\n')
    env=dict(os.environ,TMPDIR=str(E.parent/'tmp'),PYTHONDONTWRITEBYTECODE='1');env.pop('CARGO_TARGET_DIR',None)
    with (directory/'stdout.log').open('w') as out,(directory/'stderr.log').open('w') as err:
        done=subprocess.run(command,cwd=E.parent/'sifr',env=env,stdout=out,stderr=err)
    receipt['exit']=done.returncode
    if name=='coverage' and (E/'offline-result.json').exists():
        (E/'offline-result.json').rename(directory/'suite.json')
    if name=='observer':
        text=(directory/'stdout.log').read_text()
        decoder=json.JSONDecoder()
        for i,c in enumerate(text):
            if c!='{':continue
            try:report,_=decoder.raw_decode(text[i:])
            except ValueError:continue
            if isinstance(report,dict) and 'receipt' in report:
                shutil.copyfile(report['receipt'],directory/'suite.json')
                receipt['original_suite_receipt']=report['receipt']
    if (directory/'suite.json').exists():
        suite=json.loads((directory/'suite.json').read_text())
        receipt.update(state=suite['state'],count=suite['count'],
                       failures=[c for c in suite['cases'] if not c['pass']])
    else:receipt['state']='FAIL'
    (directory/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps({k:v for k,v in receipt.items() if k in ('exit','state','count','failures')},indent=2))
    print('receipt',directory/'receipt.json')
    return done.returncode

if __name__=='__main__':raise SystemExit(main())
