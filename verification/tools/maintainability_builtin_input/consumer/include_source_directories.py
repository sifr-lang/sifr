"""Exact complete directory membership with shared ancestor enumeration."""
from pathlib import Path
from include_source_authority import require


def authenticate(inventories,api):
    enumerated={}
    for name,inventory in sorted(inventories.items(),key=lambda row:len(Path(row[0]).parts)):
        root=Path(name);exclude=inventory['exclude_operational']
        parents=[(p,values) for (p,flag),values in enumerated.items() if flag==exclude and root.is_relative_to(p)]
        if parents:
            parent,values=max(parents,key=lambda row:len(row[0].parts));prefix=root.relative_to(parent).as_posix()+'/'
            actual=[v[len(prefix):] for v in values if v.startswith(prefix)]
        else:
            require(root.is_dir(),'missing original input directory: '+name,api)
            actual=sorted(str(p.relative_to(root)) for p in root.rglob('*') if p.is_file() and (not exclude or not set(p.relative_to(root).parts).intersection({'.git','target','__pycache__'})))
            enumerated[(root,exclude)]=actual
        require(actual==inventory['members'],'original directory membership drift: '+name,api)
