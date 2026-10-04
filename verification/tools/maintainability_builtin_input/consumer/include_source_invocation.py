#!/usr/bin/env python3
"""Record every actual invocation in the original normal Cargo control."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import uuid


def main():
    compiler,*args=sys.argv[1:]
    record={'schema':'sifr-maintainability-original-invocation-v1','compiler':compiler,'args':args,'cwd':os.getcwd(),'environment':dict(os.environ)}
    if 'RUSTC_BOOTSTRAP' in record['environment']:
        raise RuntimeError('bootstrap reached original normal Cargo control')
    # Cargo marks its jobserver descriptors inheritable; keep that exact channel.
    status=subprocess.run([compiler,*args],close_fds=False).returncode
    record['compiler_status']=status
    data=json.dumps(record,sort_keys=True,separators=(',',':')).encode()
    destination=Path(os.environ['SIFR_BUILTIN_ORIGINAL_INVOCATIONS'])/(str(uuid.uuid4())+'-'+hashlib.sha256(data).hexdigest()+'.json')
    temporary=destination.with_suffix('.tmp')
    with temporary.open('xb') as output:output.write(data)
    temporary.replace(destination)
    return status


if __name__=='__main__':sys.exit(main())
