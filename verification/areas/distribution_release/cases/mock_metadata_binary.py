#!/usr/bin/env python3
"""Self-contained distribution fixture; never used by production Cargo packaging."""
import hashlib
import json
from pathlib import Path
import sys

MESSAGE = "__FIXTURE_MESSAGE__"
identity = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
if sys.argv[1:] == ["--print", "compiler-identity"]:
    print(identity)
elif sys.argv[1:3] == ["sysroot", "build-metadata"]:
    arguments = dict(zip(sys.argv[3::2], sys.argv[4::2]))
    target = arguments["--target"]
    digest = hashlib.sha256()
    for name, value in [("domain", b"target-semantic-v1"), ("triple", target.encode()),
                        ("layout", b"pointer64-little-endian-v1")]:
        for part in (name.encode(), value):
            digest.update(len(part).to_bytes(8, "little"))
            digest.update(part)
    logical = (b"SIFRMETA" + (3).to_bytes(4, "little") + bytes(4)
               + bytes.fromhex(identity) + digest.digest()
               + hashlib.sha256(b"explicit-packaging-fixture-inputs").digest()
               + (120).to_bytes(8, "little"))
    # A directory frame with one raw 120-byte block, then an empty payload frame.
    # Fixtures stay self-contained and do not require a host compressor.
    frame = bytes.fromhex("28b52ffd2078c10300") + logical + bytes.fromhex("28b52ffd2000010000")
    header = (logical[:112] + (128 + len(frame)).to_bytes(8, "little")
              + len(logical).to_bytes(8, "little") + frame)
    Path(arguments["--output"]).write_bytes(header)
    print(json.dumps({"compiler_identity": identity, "semantic_target": target,
                      "metadata_id": hashlib.sha256(header).hexdigest()}))
else:
    print(MESSAGE)
