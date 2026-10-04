"""Preserve generated package inputs and compare actual rustc build recipes."""
from pathlib import Path
import hashlib
import shutil

from benchmark_manifest import BenchmarkError


def package_files(project):
    files = {}
    for path in sorted(project.rglob('*')):
        relative = path.relative_to(project)
        if relative.parts[0] == 'target':
            continue  # Cargo outputs, never package source inputs.
        if path.is_symlink() or not (path.is_dir() or path.is_file()):
            raise BenchmarkError('allocation package inputs must be regular files')
        if path.is_file() and str(relative) != 'src/main.rs':
            files[str(relative)] = hashlib.sha256(path.read_bytes()).hexdigest()
    return files


def copy_package(original, destination):
    expected = package_files(original)
    for name in expected:
        target = destination/name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(original/name, target)
    if package_files(destination) != expected:
        raise BenchmarkError('allocation package input copy differs')


def normalized_command(arguments):
    """Only relocate output/dependency paths and Cargo application hash fields."""
    normalized = []
    index = 0
    while index < len(arguments):
        argument = arguments[index]
        if argument in ('--out-dir', '-L', '--extern'):
            if index+1 == len(arguments):
                raise BenchmarkError('incomplete allocation rustc recipe')
            value = arguments[index+1]
            if argument == '--out-dir':
                value = '<output-directory>'
            elif argument == '--extern':
                name, separator, path = value.partition('=')
                if not separator or not Path(path).is_absolute():
                    raise BenchmarkError('allocation extern must identify its actual artifact')
                value = name+'='+Path(path).name
            else:
                kind, separator, path = value.partition('=')
                if kind != 'dependency' or not separator or not Path(path).is_absolute():
                    # Native linker search paths affect loading: preserve them.
                    normalized.extend((argument, value));index += 2;continue
                value = 'dependency=<output-directory>'
            normalized.extend((argument, value));index += 2;continue
        if argument == '-C' and index+1 < len(arguments):
            value = arguments[index+1]
            if value.startswith(('metadata=', 'extra-filename=')):
                normalized.extend((argument, value.split('=',1)[0]+'=<cargo-hash>'))
                index += 2;continue
        normalized.append(argument);index += 1
    return normalized


def require_same_recipe(original, actual):
    if normalized_command(original) != normalized_command(actual):
        raise BenchmarkError('instrumented rustc recipe differs from the prepared application')
