"""Bind the real generated Cargo bin artifact to its delivered executable."""
import hashlib
from pathlib import Path
import tomllib

from benchmark_manifest import BenchmarkError


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def option(arguments, name):
    values = []
    for index, argument in enumerate(arguments):
        if argument == name:
            if index + 1 == len(arguments):
                raise BenchmarkError('incomplete rustc option: '+name)
            values.append(arguments[index+1])
        elif argument.startswith(name+'='):
            values.append(argument[len(name)+1:])
    return values


def application_events(cargo_rows, rustc_rows, binary):
    expected = sha256(binary)
    artifacts = [row for row in cargo_rows if row.get('reason') == 'compiler-artifact'
                 and row.get('target', {}).get('kind') == ['bin'] and row.get('executable')
                 and sha256(row['executable']) == expected]
    if len(artifacts) != 1:
        raise BenchmarkError('exactly one actual delivered Cargo bin artifact is required')
    artifact = artifacts[0]
    manifest_path = Path(artifact['manifest_path'])
    manifest = tomllib.loads(manifest_path.read_text())
    declared = [row for row in manifest.get('bin', []) if row.get('name') == artifact['target']['name']]
    if (manifest['package']['name'] != 'sifr_output' or len(declared) != 1
            or (manifest_path.parent/declared[0]['path']).resolve() != Path(artifact['target']['src_path']).resolve()):
        raise BenchmarkError('Cargo artifact is not the actual generated application target')
    name = artifact['target']['name'].replace('-', '_')
    commands = [args for args in rustc_rows if option(args, '--crate-name') == [name]
                and option(args, '--crate-type') == ['bin']]
    if len(commands) != 1:
        raise BenchmarkError('exactly one matching generated application rustc bin command is required')
    return artifact, commands[0]
