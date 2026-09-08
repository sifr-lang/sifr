"""Per-target completion with one live session; B38 custody is replayed intact."""
import hashlib
import os
from coverage_custody import validate_completion, replay_document, ancestry, probe_row
from coverage_native import live
from coverage_output import STREAMS, MAX_STREAM, open_stream, parse_completion
from coverage_symbols import Protocol, require

def target_release(custody, group_exists):
    """Fresh native and full-table absence, with root/LLDB retained explicitly."""
    require(custody.capture_valid and custody.last is not None
            and custody.last['native']['error'] is None and not custody.rejections,
            'release custody rejected')
    debuggers = [pid for pid in custody.first if custody.role(pid) == 'debugger']
    require(len(debuggers) == 1, 'release lacks sole debugger identity')
    debugger = debuggers[0]
    keepers = {custody.root, debugger}
    require(all(live(custody.native_last[p]) and p not in custody.tainted for p in keepers),
            'session/root identity lost before release')
    require(custody.targets and len(custody.targets) == 1, 'release lacks sole target')
    require(custody.native_last[debugger]['after']['ppid'] == custody.root,
            'debugger escaped original root')
    rows = custody.last['rows']
    groups = custody.groups - {debugger}
    chains = ancestry(rows, custody.root)
    remaining = [r for r in rows if r['pid'] not in keepers and not probe_row(custody.last, r)
        and (r['pid'] in custody.native_initial or r['pgid'] in groups or r['pid'] in chains)]
    absent = not remaining and all(not e['present'] for p, e in custody.native_last.items() if p not in keepers)
    absent = absent and not any(group_exists(g) for g in groups)
    return {'state': 'PASS' if absent else 'PENDING', 'run_id': custody.run_id,
        'target_ids': sorted(custody.targets), 'keepers': sorted(keepers),
        'remaining': remaining, 'groups': sorted(groups), 'groups_absent': absent,
        'native_sequence': custody.last['native']['sequence'],
        'observation_index': len(custody.observations) - 1}

def complete(route, observer, handshake, ack, custody, release):
    pid, rid = observer['pid'], route['run_id']
    validate_completion(ack, custody, pid, rid, route['binary'])
    replayed = replay_document(custody, route['binary'], rid)
    require(observer['state'] == 'ENTRIES_COMPLETE' and observer['stage'] == 'completion'
            and 'error' not in observer and observer.get('target_deleted') is True, 'observer not complete')
    require(observer['run_id'] == handshake['run_id'] == rid and observer['target_launches'] == 1,
            'wrong per-target identity')
    require(handshake['pid'] == handshake['pgid'] == pid and handshake['flags'] == 134,
            'invalid per-target handshake')
    require(observer['exit'] == 0 and observer['exited'] is True, 'target exit invalid')
    expected = [{'fd': fd, 'path': route['streams'][name]['path'], 'read': False, 'write': True}
                for fd, name in STREAMS]
    require(observer['output_actions'] == expected, 'output fd provenance mismatch')
    require(release['state'] == 'PASS' and release['groups_absent'] is True
            and release['remaining'] == [] and release['target_ids'] == [pid]
            and release['run_id'] == rid, 'target release missing')
    # The producer recorded actual group queries. Replay native/ps and the
    # exact keeper contract without issuing any process probe in the consumer.
    expected_release = target_release(replayed, lambda g: not release['groups_absent'])
    require(release == expected_release, 'target release does not match fresh custody')
    raw, streams = {}, {}
    for fd, name in STREAMS:
        saved = route['streams'][name]
        handle, before = open_stream(saved)
        with os.fdopen(handle, 'rb') as stream:
            require(before.st_size <= MAX_STREAM, 'stream size exceeded')
            data = stream.read(MAX_STREAM + 1)
            after = os.fstat(stream.fileno())
        require(len(data) == before.st_size == after.st_size and before.st_mtime_ns == after.st_mtime_ns,
                'output changed during read')
        raw[name] = data
        streams[name] = dict(saved, size=len(data), sha256=hashlib.sha256(data).hexdigest())
    checks = parse_completion(raw['stdout'], raw['stderr'])
    protocol = Protocol()
    protocol.phase, protocol.handover_count = observer['final_phase'], observer['handover_count']
    protocol.hits = observer['hits']
    protocol.finish(observer['exit'], len(checks))
    return {'state': 'PASS', 'pid': pid, 'run_id': rid, 'streams': streams,
            'file_check_phases': len(checks), 'release_native_sequence': release['native_sequence'],
            'canonical_output_after_target_release': True, 'session_still_owned': True}
