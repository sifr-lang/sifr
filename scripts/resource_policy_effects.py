"""Bounded Buffer/Arrow/DLPack effects; source contracts still require review."""
from rust_policy_sites import code_tokens

RESOURCE_RULES = {
    'resource-release': ('resource-release-attached', 'resource-live-until-terminal-release',
                         'exclusive-terminal-resource-operation', 'consume-resource-release-once'),
    'resource-capsule-transfer': ('capsule-transfer-attached', 'capsule-and-payload-pinned-through-transfer',
                                  'one-shot-capsule-transfer', 'transfer-capsule-release-authority'),
    'resource-header-initialize': ('resource-header-caller', 'writable-initialized-size-header',
                                   'exclusive-output-header-write', 'initialize-output-release-owner'),
    'buffer-export-acquire': ('supported-gil', 'pinned-view-through-export',
                              'exclusive-export-output', 'acquire-exporter-reference'),
    'buffer-element-read': ('supported-gil', 'exporter-pins-admitted-element',
                            'serialized-admitted-read', 'copy-value-preserve-export-owner'),
    'buffer-element-write': ('supported-gil', 'exporter-pins-admitted-element',
                             'serialized-exclusive-writable-view', 'write-value-preserve-export-owner'),
}


def resource_operation_family(site):
    """Strong transfer effects precede declaration and observation refinements.

    Restricted to the source resource partition. Nullable field observations
    are borrows; invoking a release callback or retiring its marker consumes
    authority. Unrecognized operations retain the foundation's raw admission.
    """
    from unsafe_policy_segments import segment_for
    if segment_for(site) != 'python_resources':
        return None
    ts = [t.value for t in code_tokens(site.text)]
    identifiers = set(ts)
    calls = {ts[i] for i in range(len(ts) - 1) if ts[i + 1] == '('}
    if ('from_raw' in calls or calls & {'deleter', 'legacy_deleter', 'versioned_deleter',
                                      'test_deleter', 'callback', 'release_schema', 'release_array',
                                      'release_stream', 'release_device_stream'}
            or ('release' in calls and site.kind != 'unsafe-impl')
            or any(ts[i:i + 4] == ['release', '=', 'None', '}'] or
                   ts[i:i + 4] == ['release', '=', 'None', ';']
                   for i in range(len(ts) - 3))):
        return 'resource-release'
    if identifiers & {'PyCapsule_SetName', 'new_with_pointer_and_destructor'}:
        return 'resource-capsule-transfer'
    if 'PyObject_GetBuffer' in identifiers:
        return 'buffer-export-acquire'
    if 'write_unaligned' in calls:
        return 'buffer-element-write'
    if 'read_unaligned' in calls and site.path.endswith('/buffer_ops/access.rs'):
        return 'buffer-element-read'
    if any(ts[i:i + 3] == ['output', '.', 'write'] for i in range(len(ts) - 2)):
        return 'resource-header-initialize'
    return None
