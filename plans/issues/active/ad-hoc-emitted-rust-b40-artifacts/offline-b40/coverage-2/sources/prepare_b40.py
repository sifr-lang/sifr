"""One-time owned copies and reversible relocation; no predecessor writes."""
import hashlib
import json
import shutil
from pathlib import Path

E = Path(__file__).resolve().parent
OLD = Path('/private/tmp/sifr-b39.20ZQBg/evidence')
HOST = Path('/private/tmp/sifr-b39-host.8IkfrA/evidence')

def digest(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def save(p, value):
    p.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')

def copy_exact(source, destination):
    if Path(destination).exists():
        assert digest(source) == digest(destination), str(destination)
        return destination
    return shutil.copy2(source, destination)

def main():
    assert digest(HOST / 'terminal.json') == 'be21459548a64ad31aefa3fbea327f961b194e6429a8694335db93b070d287ef'
    assert digest(OLD / 'terminal.json') == 'cea8e3fe6265971beeddb8e03d6c650a2b2f534910620e3d40dd4d76bbefd944'
    mappings = {}
    for source in (HOST, OLD):
        receipt = json.loads((source / 'terminal.json').read_text())
        hashes = receipt.get('evidence_sha256', receipt.get('file_hashes', {}))
        assert hashes, list(receipt)
        for path, expected in hashes.items():
            assert digest(path) == expected, path
        print('authenticated', source, len(hashes), flush=True)
    shutil.copytree(HOST, E / 'prior-b39-host', dirs_exist_ok=True, copy_function=copy_exact)
    for path, expected in json.loads((HOST / 'terminal.json').read_text())['evidence_sha256'].items():
        p = Path(path)
        if p.is_relative_to(HOST):
            owned = E / 'prior-b39-host' / p.relative_to(HOST)
        else:
            owned = E / 'original-records' / p.name
            owned.parent.mkdir(exist_ok=True)
            copy_exact(p, owned)
        assert digest(owned) == expected
        mappings[path] = {'path': str(owned), 'sha256': expected}
    active = list(OLD.glob('*.py')) + [OLD / 'boundaries.lldb', OLD / 'coverage.lldb']
    relocated = {}
    for p in active:
        before = p.read_bytes()
        after = before.replace(str(OLD).encode(), str(E).encode()).replace(
            b'12K-B39-20ZQBg-target-', b'12K-B40-urwpgv-target-')
        assert after.replace(str(E).encode(), str(OLD).encode()).replace(
            b'12K-B40-urwpgv-target-', b'12K-B39-20ZQBg-target-') == before
        dest = E / p.name
        dest.write_bytes(after)
        relocated[p.name] = {'original': str(p), 'before': digest(p),
                             'after': digest(dest), 'inverse_equal': True}
    for name in ('sifr-experiment09', 'identity-contract.json', 'counter-layout.json',
                 'counter-units.md', 'module-inventory.json'):
        copy_exact(OLD / name, E / name)
    shutil.copytree(OLD / 'inputs', E / 'inputs', dirs_exist_ok=True, copy_function=copy_exact)
    for name in ('prior-b37-cont/failure-evidence.json',
                 'prior-b24/boundaries/target-0/final-custody.json',
                 'prior-b24/complete-release-record.json',
                 'prior-b24/boundaries/target-0/events.json'):
        dest = E / name
        dest.parent.mkdir(parents=True, exist_ok=True)
        copy_exact(OLD / name, dest)
    # Saved analyzer replay needs the entire small target directory, not a new run.
    shutil.copytree(OLD / 'prior-b24/boundaries/target-0', E / 'prior-b24/boundaries/target-0', dirs_exist_ok=True, copy_function=copy_exact)
    save(E / 'preservation-map.json', mappings)
    save(E / 'relocation.json', relocated)
    print('copied', len(mappings), 'active', len(active), 'bytes', sum(p.stat().st_size for p in E.rglob('*') if p.is_file()), flush=True)

if __name__ == '__main__':
    main()
