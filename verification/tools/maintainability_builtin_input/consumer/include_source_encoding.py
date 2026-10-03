"""Canonical JSON digests without simultaneous whole-string/byte copies."""
import hashlib
import json
from pathlib import Path


def chunks(value):
    for chunk in json.JSONEncoder(sort_keys=True,separators=(',',':')).iterencode(value):
        yield chunk.encode()


def digest(value):
    result=hashlib.sha256()
    for chunk in chunks(value):result.update(chunk)
    return result.hexdigest()


def write(path,value):
    path=Path(path);temporary=path.with_suffix(path.suffix+'.tmp')
    with temporary.open('xb') as output:
        for chunk in chunks(value):output.write(chunk)
    temporary.replace(path)
