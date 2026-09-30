"""Bounded Buffer/Arrow/DLPack effects; source contracts still require review."""
from rust_policy_sites import code_tokens, pairs

RESOURCE_RULES = {
    'resource-release': ('resource-release-attached', 'resource-live-until-terminal-release',
                         'exclusive-terminal-resource-operation', 'consume-resource-release-once'),
    'resource-capsule-transfer': ('capsule-transfer-attached', 'capsule-and-payload-pinned-through-transfer',
                                  'one-shot-capsule-transfer', 'transfer-capsule-release-authority'),
    'resource-reference-acquire': ('resource-reference-attached', 'resource-reference-live-through-acquisition',
                                   'exclusive-resource-reference-transfer', 'acquire-or-transfer-owned-resource-reference'),
    'resource-header-initialize': ('resource-header-caller', 'writable-initialized-size-header',
                                   'exclusive-output-header-write', 'initialize-output-release-owner'),
    'buffer-export-acquire': ('supported-gil', 'pinned-view-through-export',
                              'exclusive-export-output', 'acquire-exporter-reference'),
    'buffer-element-read': ('supported-gil', 'exporter-pins-admitted-element',
                            'serialized-admitted-read', 'copy-value-preserve-export-owner'),
    'buffer-element-write': ('supported-gil', 'exporter-pins-admitted-element',
                             'serialized-exclusive-writable-view', 'write-value-preserve-export-owner'),
}


def release_slot_bindings(context):
    """Names extracted from a release field in this fingerprinted context.

    This is bounded lexical provenance, not a Rust dataflow/type proof. The
    current records additionally bind the actual nullable admission and call.
    """
    ts = code_tokens(context)
    ps = pairs(ts)
    result = set()
    for i, token in enumerate(ts):
        if token.value != 'let' or i + 3 >= len(ts):
            continue
        j = i + 1
        if ts[j].value == 'mut':
            j += 1
        if ts[j].kind != 'ident' or ts[j + 1].value != '=':
            continue
        name, start = ts[j].value, j + 2
        end = start
        while end < len(ts) and ts[end].value != ';':
            end = ps[end] + 1 if ts[end].value in ('(', '[', '{') else end + 1
        values = [t.value for t in ts[start:end]]
        if any(values[k:k + 2] == ['.', 'release'] for k in range(len(values) - 1)):
            result.add(name)
    return result


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
    if calls & release_slot_bindings(site.context_text):
        return 'resource-release'
    if ('from_raw' in calls or calls & {'deleter', 'legacy_deleter', 'versioned_deleter',
                                      'test_deleter', 'callback', 'release_schema', 'release_array',
                                      'release_stream', 'release_device_stream'}
            or ('release' in calls and site.kind != 'unsafe-impl')
            or any(ts[i:i + 4] == ['release', '=', 'None', '}'] or
                   ts[i:i + 4] == ['release', '=', 'None', ';']
                   for i in range(len(ts) - 3))):
        return 'resource-release'
    if identifiers & {'into_ptr', 'from_owned_ptr_or_err'}:
        return 'resource-reference-acquire'
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
