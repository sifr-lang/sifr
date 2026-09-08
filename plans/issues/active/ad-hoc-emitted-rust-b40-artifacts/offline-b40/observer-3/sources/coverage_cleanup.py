"""B35 cleanup scheduling: B34-F1 stable snapshots, bounded fresh discovery.

Scheduling never grants signal authority. Each kill callback still invokes the
unchanged Custody.cleanup_allowed against a fresh authenticated observation.
"""
from coverage_custody import ancestry, probe_row


def signal_targets(custody, kill_group):
    for pid in tuple(sorted(custody.targets)):
        kill_group(pid)


def drain(custody, inspect, kill_group, group_exists, until, monotonic, sleep):
    """New roles discovered while signaling join later bounded passes."""
    while monotonic() < until:
        inspect()
        signal_targets(custody, kill_group)
        for group in tuple(sorted(custody.groups)):
            kill_group(group)
        # Discovery cannot depend solely on the groups in the earlier snapshot.
        inspect()
        if not any(group_exists(g) for g in tuple(sorted(custody.groups))):
            break
        sleep(0.05)


def final_absence(custody, capture, group_exists):
    """A final fresh full observation, never a prior scheduling snapshot."""
    current = custody.observe(capture())
    groups = tuple(sorted(custody.groups))
    remaining = [r for r in current if
                 r['pid'] != custody.root and not probe_row(custody.last, r) and
                 (r['pid'] in custody.native_initial or r['pgid'] in groups
                  or r['command'] == custody.binary or r['pid'] in ancestry(current, custody.root))]
    native_absent = all(not custody.native_last.get(pid, {}).get('present', True)
                        for pid in custody.native_initial if pid != custody.root)
    absent = (custody.capture_valid and custody.last.get('native', {}).get('error', 'missing') is None
              and native_absent and not remaining
              and not any(group_exists(g) for g in groups))
    return remaining, absent
