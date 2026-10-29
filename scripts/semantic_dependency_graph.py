"""Transitive semantic/native boundaries, including Cargo's resolved edge kinds."""
from collections import deque
import json
from pathlib import Path
import subprocess

COMPILER_CRATES = {
    "sifr",
    "sifr_analysis",
    "sifr_codegen",
    "sifr_generated_rust",
    "sifr_diagnostics",
    "sifr_driver",
    "sifr_format",
    "sifr_frontend",
    "sifr_ir",
    "sifr_identity",
    "sifr_ipc",
    "sifr_lint",
    "sifr_lowering",
    "sifr_lsp",
    "sifr_metadata",
    "sifr_semantic_services",
    "sifr_package",
    "sifr_runtime",
    "sifr_source",
    "sifr_stdlib",
    "sifr_stdlib_artifact",
    "sifr_stdlib_imports",
    "sifr_stdlib_manifest",
    "sifr_sysroot",
    "sifr_syntax",
    "sifr_type_system",
}

SQL = ('sifr_sql_sqlite_tools', 'sifr_sql_mysql_tools', 'sifr_sql_postgresql_tools')
LOWER = ('sifr_sysroot', 'sifr_stdlib_manifest', 'sifr_lowering', 'sifr_codegen', 'sifr_frontend')
NATIVE = frozenset({'sifr', 'sifr_driver', 'sifr_compiler_services', 'sifr_codegen', 'sifr_stdlib_artifact'})


def semantic_rule(rule):
    forbidden = NATIVE | {'sifr_analysis', 'sifr_lsp'}
    return rule(crate='sifr_semantic_services', forbidden_all_dependencies=forbidden,
                forbidden_source_references=forbidden - {'sifr'}, scan_manifest_targets=True)


def violations(names, edges, roots, forbidden):
    failures = []
    for root in roots:
        starts = [identity for identity, name in names.items() if name == root]
        for start in starts:
            pending = deque([(start, [root])])
            seen = {start}
            while pending:
                identity, chain = pending.popleft()
                for child in sorted(edges.get(identity, ())):
                    if child in seen:
                        continue
                    seen.add(child)
                    path = chain + [names.get(child, child)]
                    if names.get(child) in forbidden:
                        failures.append('forbidden transitive dependency: ' + ' -> '.join(path))
                    pending.append((child, path))
    return failures


def manifest_failures(root: Path, guard):
    manifests = {p.parent.name: guard.load_manifest(root, p.parent.name)
                 for p in (root / 'crates').glob('*/Cargo.toml')}
    names = {name: manifest.get('package', {}).get('name', name)
             for name, manifest in manifests.items()}
    def edges(include_dev):
        return {name: {dep for values in guard.dependency_entries(manifest, root, include_dev=include_dev).values() for dep in values}
                for name, manifest in manifests.items()}
    # Include optional and every target-specific manifest edge, even when the
    # current host's resolved graph does not activate them.
    return (violations(names, edges(False), SQL, NATIVE)
            + violations(names, edges(True), ('sifr_semantic_services',), NATIVE)
            + violations(names, edges(True), LOWER, {'sifr_metadata'}))


def resolved_failures(metadata):
    names = {p['id']: p['name'] for p in metadata['packages']}
    nodes = metadata.get('resolve', {}).get('nodes', [])
    all_edges = {n['id']: {d['pkg'] for d in n['deps']} for n in nodes}
    normal_edges = {n['id']: {d['pkg'] for d in n['deps']
                            if any(k['kind'] in (None, 'build') for k in d['dep_kinds'])}
                    for n in nodes}
    return (violations(names, normal_edges, SQL, NATIVE)
            + violations(names, all_edges, ('sifr_semantic_services',), NATIVE)
            + violations(names, all_edges, LOWER, {'sifr_metadata'}))


def validate_resolved(root):
    try:
        run = subprocess.run(['cargo', 'metadata', '--locked', '--offline', '--format-version', '1'],
                             cwd=root, capture_output=True, text=True, timeout=120, check=True)
        metadata = json.loads(run.stdout)
    except (OSError, subprocess.SubprocessError, ValueError) as error:
        return [f'cannot verify resolved semantic dependency graph: {error}']
    if not metadata.get('resolve'):
        return ['resolved semantic dependency graph is missing']
    present = {p['name'] for p in metadata['packages']}
    if not set(SQL + LOWER + ('sifr_semantic_services',)) <= present:
        return ['resolved semantic dependency graph lacks required owners']
    return resolved_failures(metadata)


def self_test():
    # Synthetic resolved nodes retain aliases, target predicates and edge kinds.
    names = list(SQL) + ['sifr_semantic_services', 'sifr_frontend', 'sifr_lowering',
                         'sifr_codegen', 'sifr_metadata', 'helper']
    ids = {name: f'path+file:///fixture/{name}#0.0.0' for name in names}
    data = {'packages': [{'id': ids[n], 'name': n} for n in names],
            'resolve': {'nodes': [{'id': ids[n], 'deps': []} for n in names]}}
    nodes = {names[i]: n for i, n in enumerate(data['resolve']['nodes'])}
    def edge(source, dest, kind=None, target=None):
        nodes[source]['deps'].append({'name': 'renamed_dependency', 'pkg': ids[dest],
                                      'dep_kinds': [{'kind': kind, 'target': target}]})
    for tool in SQL:
        edge(tool, 'sifr_semantic_services')
    edge('sifr_semantic_services', 'sifr_frontend')
    edge('sifr_frontend', 'sifr_lowering')
    assert not resolved_failures(data)
    edge('sifr_semantic_services', 'helper')
    edge('helper', 'sifr_codegen', 'build', 'cfg(windows)')
    assert len(resolved_failures(data)) == 4
    nodes['helper']['deps'].clear()
    edge('sifr_lowering', 'helper', 'dev')
    edge('helper', 'sifr_metadata', 'dev', 'cfg(unix)')
    assert len(resolved_failures(data)) == 2
    nodes['helper']['deps'].clear()
    edge(SQL[0], 'sifr_codegen', 'dev')
    assert not resolved_failures(data)
    nodes[SQL[0]]['deps'][-1]['dep_kinds'].append({'kind': None, 'target': None})
    assert len(resolved_failures(data)) == 1
    return []
