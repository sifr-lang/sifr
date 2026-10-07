"""Check both restored migration residue and the intact published predecessor."""
from pathlib import Path
from published_installation import verify_installed
from published_predecessor import digest


def rollback_residue(managed, root, receipt_sha256):
    root=Path(root)
    if (root.parent!=managed/'.sifr-generations' or not root.name.startswith('legacy.')
            or root.resolve(strict=True)!=root or not root.is_dir()
            or {p.name for p in root.iterdir()}!={'install.json','bin'}
            or (root/'bin').is_symlink() or not (root/'bin').is_dir()
            or any((root/'bin').iterdir()) or (root/'install.json').is_symlink()
            or digest(root/'install.json')!=receipt_sha256):
        raise ValueError('rolled-back legacy migration residue differs')
    return root


def predecessor(managed, rolled_back, receipt_sha256, rows):
    rolled_back=rollback_residue(managed,rolled_back,receipt_sha256)
    roots=set((managed/'.sifr-generations').glob('legacy.*'))
    remaining=roots-{rolled_back}
    if len(remaining)!=1:
        raise ValueError('migration did not retain exactly one intact published payload')
    root=remaining.pop()
    if root.resolve(strict=True)!=root or digest(root/'install.json')!=receipt_sha256:
        raise ValueError('retained published predecessor receipt differs')
    verify_installed(root,rows)
    return root
