"""B34 ps observation contract. Diagnostics never grant new signal authority."""
import copy
import math
import os
import subprocess
import time
from coverage_native import checked_record, live, DEBUGGER, DEBUGSERVER

SCHEMA = '12K-B36-custody-v1'
COMMAND = ['/bin/ps', '-ww', '-axo', 'pid=,ppid=,pgid=,lstart=,stat=,comm=']
IDENTITY_FIELDS = ('pid', 'pgid', 'created', 'command', 'ppid')
COMPARED_FIELDS = IDENTITY_FIELDS + ('stat',)
STATES = frozenset('IRSTUZ')


def require(ok, reason):
    if not ok:
        raise ValueError(reason)


def same(first, current):
    return all(first.get(k) == current.get(k) and k in first and k in current
               for k in IDENTITY_FIELDS)


def lifecycle(first, current):
    """ps fields support observations, never exec/PID-reuse causal attribution."""
    if first is None or current is None:
        return {'classification': 'UNKNOWN_MISSING_ROW', 'cause': 'UNKNOWN'}
    changed = [k for k in COMPARED_FIELDS if first.get(k) != current.get(k)]
    state = current.get('stat', '')[:1]
    if state not in STATES:
        kind = 'UNKNOWN_PROCESS_STATE'
    elif first.get('created') != current.get('created'):
        kind = 'START_FIELD_CHANGED_REPLACEMENT_SUSPECTED'
    elif state == 'Z':
        kind = 'ZOMBIE_OBSERVED'
    elif 'command' in changed:
        kind = 'COMMAND_CHANGED'
    elif 'ppid' in changed or 'pgid' in changed:
        kind = 'ANCESTRY_OR_GROUP_CHANGED'
    elif changed:
        kind = 'STATE_CHANGED' if changed == ['stat'] else 'IDENTITY_CHANGED'
    else:
        kind = 'UNCHANGED_RECORDED_FIELDS'
    return {'classification': kind, 'cause': 'UNKNOWN', 'changed_fields': changed,
            'same_recorded_identity': same(first, current),
            'native_process_identity': 'UNPROVEN_BY_PS'}


def parse_rows(raw):
    result = []
    for line in raw.splitlines():
        if not line.strip():
            continue
        fields = line.strip().split(None, 9)
        require(len(fields) == 10, 'malformed ps row')
        pid, ppid, pgid = (int(value) for value in fields[:3])
        require(pid > 0 and ppid >= 0 and pgid >= 0, 'invalid ps identifiers')
        result.append({'pid': pid, 'ppid': ppid, 'pgid': pgid,
                       'created': ' '.join(fields[3:8]), 'stat': fields[8],
                       'command': fields[9], 'raw': line})
    require(len({r['pid'] for r in result}) == len(result), 'duplicate ps PID')
    return result


def run_probe(command, text, capture_output, timeout, env):
    """Own the exact probe handle/PID, including timeout cleanup."""
    with subprocess.Popen(command, text=text, stdout=subprocess.PIPE,
                          stderr=subprocess.PIPE, env=env) as probe:
        try:
            stdout, stderr = probe.communicate(timeout=timeout)
        except subprocess.TimeoutExpired:
            probe.kill()
            probe.communicate()
            raise
        result = subprocess.CompletedProcess(command, probe.returncode, stdout, stderr)
        result.probe_pid = probe.pid
        return result


def capture(run=run_probe, monotonic=time.monotonic, wall=time.time):
    """Bracket one non-atomic process table observation; preserve failures/raw."""
    receipt = {'command': list(COMMAND), 'clock_owner': 'outer Python',
               'begin_monotonic': monotonic(), 'begin_wall': wall(),
               'rows': [], 'raw': None, 'error': None}
    try:
        completed = run(COMMAND, text=True, capture_output=True, timeout=2,
                        env=dict(os.environ, LC_ALL='C', TZ='UTC'))
        receipt.update(raw=completed.stdout, stderr=completed.stderr,
                       probe_pid=getattr(completed, 'probe_pid', None),
                       returncode=completed.returncode)
        require(completed.returncode == 0, 'ps command failed')
        receipt['rows'] = parse_rows(completed.stdout)
    except Exception as exc:
        receipt['error'] = repr(exc)
    receipt.update(end_monotonic=monotonic(), end_wall=wall())
    return receipt


def validate_snapshot(snapshot):
    require(snapshot['command'] == COMMAND and snapshot['clock_owner'] == 'outer Python',
            'wrong observation producer')
    require(snapshot['error'] is None and snapshot['returncode'] == 0, 'capture failed')
    for key in ('begin_monotonic', 'end_monotonic', 'begin_wall', 'end_wall'):
        value = snapshot[key]
        require(type(value) in (int, float) and math.isfinite(value), 'invalid timestamp')
    require(snapshot['end_monotonic'] >= snapshot['begin_monotonic'], 'reversed observation')
    require(snapshot['rows'] == parse_rows(snapshot['raw']), 'raw/parsed rows disagree')
    pid = snapshot.get('probe_pid')
    require(pid is None or type(pid) is int and pid > 0, 'invalid probe PID')


def probe_row(snapshot, row):
    return row['pid'] == snapshot.get('probe_pid') and row['command'] == '/bin/ps'


def observed_rows(snapshot):
    from coverage_acquisition import confirmed_departures
    departed = confirmed_departures(snapshot)
    return [r for r in snapshot['rows'] if r['pid'] not in departed]


def ancestry(rows, root):
    """Observed chain only; a process-table read is not an atomic OS identity."""
    by_pid = {r['pid']: r for r in rows}
    chains = {}
    for row in rows:
        chain, pid = [], row['pid']
        while pid != root and pid in by_pid and pid not in chain:
            chain.append(pid)
            pid = by_pid[pid]['ppid']
        if pid == root:
            chains[row['pid']] = chain + [root]
    return chains


class Custody:
    def __init__(self, root, parent_group, binary, run_id):
        self.root, self.parent_group = root, parent_group
        self.binary, self.run_id = str(binary), run_id
        self.first = {}
        self.observations = []
        self.rejections = []
        self.groups, self.targets = set(), set()
        self.tainted = set()
        self.last = None
        self.capture_valid = False
        self.native_initial, self.native_last = {}, {}
        self.departed, self.transitions = set(), []
        self.accepted_parents = {}
        self.acquired_chains = {}

    def role(self, pid):
        native = self.native_initial.get(pid)
        if native is None:
            return None
        path = native['registration']['after']['path']
        return ('inferior' if path == self.binary else 'debugger' if path == DEBUGGER
                else 'debugserver' if path == DEBUGSERVER else None)

    def matches(self, pid, row):
        first = self.first[pid]['row']
        native = self.native_last[pid]['after']
        display = (row['command'] == first['command'] or row['command'] == native['path']
                   or row['command'] == '(' + native.get('comm', '') + ')')
        return (all(first[k] == row[k] for k in ('pid', 'pgid', 'created'))
                and display and row['ppid'] in self.accepted_parents.get(pid, {first['ppid']}))

    def authenticate_native(self, snapshot):
        rows = {r['pid']: r for r in snapshot['rows']}
        chains = ancestry(snapshot['rows'], self.root)
        subjects = set(self.native_initial) | {self.root} | {
            pid for pid in chains if not probe_row(snapshot, rows[pid])}
        for pid in sorted(subjects):
            try:
                entry = checked_record(snapshot, pid, self.native_last.get(pid),
                                       allow_absent=pid != self.root)
                if not entry['present']:
                    self.departed.add(pid)
                else:
                    require(pid not in self.departed, 'departed PID returned')
                if pid not in self.native_initial:
                    if not entry['present'] and entry.get('acquisition_departure'):
                        self.native_last[pid] = copy.deepcopy(entry)
                        continue  # diagnostic absence never becomes a signal identity
                    require(live(entry), 'initial native subject absent or terminated')
                    self.native_initial[pid] = copy.deepcopy(entry)
                self.native_last[pid] = copy.deepcopy(entry)
            except (ValueError, KeyError, TypeError) as exc:
                self.reject('native custody: ' + str(exc), rows.get(pid), snapshot)
                self.tainted.add(pid)

    def transition(self, pid, row, rows):
        """Only attach to original LLDB's child, or return to that same LLDB."""
        first = self.first[pid]['row']
        if row['ppid'] in self.accepted_parents.get(pid, {first['ppid']}):
            previous = next((r for r in self.observations[-2]['rows'] if r['pid'] == pid), None) \
                if len(self.observations) > 1 else None
            if previous and previous['ppid'] != row['ppid']:
                self.transitions.append({'pid': pid, 'from_parent': previous['ppid'],
                    'to_parent': row['ppid'], 'observation_index': len(self.observations) - 1,
                    'kind': 'supported-return-or-reattach-observation', 'historical_cause': 'UNKNOWN'})
            return
        original, new = first['ppid'], row['ppid']
        require(self.role(pid) == 'inferior' and self.role(original) == 'debugger'
                and self.role(new) == 'debugserver', 'unsupported custody parent transition')
        require(new in self.first and rows.get(new, {}).get('ppid') == original
                and self.first[new]['row']['ppid'] == original,
                'new parent is not original debugger child')
        require(all(p in rows and p not in self.tainted and p not in self.departed
                    for p in (pid, original, new, self.root)), 'transition parent identity unproved')
        require(self.matches(original, rows[original]) and self.matches(new, rows[new])
                and rows[original]['ppid'] == self.root, 'transition parents escaped original chain')
        require(all(first[k] == row[k] for k in ('pid', 'pgid', 'created', 'command')),
                'transition subject identity changed')
        self.accepted_parents[pid] = {original, new}
        self.transitions.append({'pid': pid, 'original_parent': original, 'tracer_parent': new,
            'observation_index': len(self.observations) - 1,
            'kind': 'supported-attach-chain-observation', 'historical_cause': 'UNKNOWN'})

    def document(self):
        return copy.deepcopy({'schema': SCHEMA, 'run_id': self.run_id,
            'root_pid': self.root, 'parent_group': self.parent_group,
            'processes': [x['row'] for x in self.first.values()],
            'initial_observations': self.first, 'observations': self.observations,
            'owned_groups': sorted(self.groups), 'target_ids': sorted(self.targets),
            'tainted_pids': sorted(self.tainted), 'rejections': self.rejections,
            'native_initial': self.native_initial, 'departed_pids': sorted(self.departed),
            'transitions': self.transitions, 'acquired_chains': self.acquired_chains,
            'state': 'REJECTED' if self.rejections else 'ACTIVE'})

    def reject(self, reason, row=None, snapshot=None):
        pid = row['pid'] if row else None
        first = self.first.get(pid)
        # Retain the first conflict for each reason/PID, never replace its clocks.
        if any(r['reason'] == reason and r['pid'] == pid for r in self.rejections):
            return
        if pid is not None:
            self.tainted.add(pid)
        current = snapshot if snapshot is not None else self.last
        rows = current.get('rows', []) if current else []
        try:
            chains = ancestry(rows, self.root)
        except (KeyError, TypeError):
            chains = None
        self.rejections.append(copy.deepcopy({'reason': reason, 'pid': pid,
            'first': first, 'conflicting_row': row, 'observation': current,
            'compared_fields': list(COMPARED_FIELDS),
            'comparison': {k: {'first': first['row'].get(k) if first else None,
                               'current': row.get(k) if row else None}
                           for k in COMPARED_FIELDS},
            'lifecycle': lifecycle(first['row'] if first else None, row),
            'ancestry': chains,
            'owned_groups': sorted(self.groups), 'target_ids': sorted(self.targets),
            'initial_signal_identities_preserved': True, 'custody_state': 'REJECTED'}))

    def observe(self, snapshot):
        """Commit accepted state only after every receipt/identity check passes."""
        trial = copy.deepcopy(self)
        rows = trial._observe(snapshot)
        if trial.rejections:
            self.capture_valid = False
            self.rejections = trial.rejections
            self.tainted.update(trial.tainted)
            # Raw failed attempts stay in rejection receipts, without becoming
            # the accepted last snapshot, native identity or owned-group state.
            return []
        self.__dict__.update(trial.__dict__)
        return rows

    def _observe(self, snapshot):
        snapshot = copy.deepcopy(snapshot)
        self.capture_valid = False
        try:
            validate_snapshot(snapshot)
            if self.last:
                require(snapshot['begin_monotonic'] > self.last['end_monotonic'],
                        'stale or overlapping observation')
        except (ValueError, KeyError, TypeError) as exc:
            self.reject('invalid observation: ' + str(exc), snapshot=snapshot)
            return []
        self.last = snapshot
        self.observations.append(snapshot)
        try:
            rows = observed_rows(snapshot)
            from coverage_acquisition import initial_chain
            acquired = initial_chain(snapshot, self.root, self.binary)
            require(all(pid not in self.first and pid not in self.native_initial for pid in acquired),
                    'initial acquisition cannot replace accepted target identity')
        except (ValueError, KeyError, TypeError) as exc:
            self.reject('invalid acquisition: ' + str(exc), snapshot=snapshot)
            return []
        self.capture_valid = True
        chains = ancestry(rows, self.root)
        by_pid = {r['pid']: r for r in rows}
        self.authenticate_native(snapshot)
        # A vanished foreign group member remains a recorded contradiction.
        for row in snapshot['rows']:
            if (row not in rows and row['pgid'] in self.groups
                    and row['pid'] not in self.first and not probe_row(snapshot, row)):
                self.reject('foreign vanished owned-group member', row)
        for row in rows:
            pid = row['pid']
            chain = chains.get(pid)
            if (pid != self.root and pid not in self.first and chain and not self.rejections
                    and self.role(pid) is not None):
                self.first[pid] = {'row': copy.deepcopy(row),
                    'observation_index': len(self.observations) - 1,
                    'observation': copy.deepcopy(snapshot), 'ancestry': chain}
                self.accepted_parents[pid] = {row['ppid']}
                if pid in acquired:
                    chain = acquired[pid]
                    self.accepted_parents[pid] = {p for p in (chain['debugger'], chain['tracer'])
                                                  if p is not None}
                    self.acquired_chains[pid] = dict(chain,
                        observation_index=len(self.observations) - 1,
                        kind='coherent-initial-acquisition', historical_cause='UNKNOWN')
        # Inspect known identities even after group escape/reparenting. Check
        # old identities before admitting any new descendant or group.
        for pid, first in list(self.first.items()):
            row = by_pid.get(pid)
            if row is None:
                continue  # Disappearance is recorded, never classified as exit.
            try:
                self.transition(pid, row, by_pid)
            except (ValueError, KeyError, TypeError) as exc:
                self.reject(str(exc), row)
            if not self.matches(pid, row):
                self.reject('recorded process identity changed', row)
            terminal = self.terminal_departure(pid, row, by_pid)
            if len(self.accepted_parents.get(pid, ())) > 1 and any(
                    p not in by_pid or p in self.tainted or p in self.departed
                    for p in self.accepted_parents[pid]) and not terminal:
                self.reject('transition parent no longer authenticated', row)
            if pid not in chains and not terminal:
                self.reject('authenticated ancestry lost', row)
            if row['stat'][:1] not in STATES:
                self.reject('unknown lifecycle state', row)
        for row in rows:
            pid = row['pid']
            if pid == self.root:
                continue
            chain = chains.get(pid)
            if pid in self.first and row['stat'][:1] not in STATES:
                self.reject('unknown lifecycle state', row)
            role = self.role(pid)
            if role and (chain or pid in self.first):
                if role == 'inferior':
                    self.targets.add(pid)
                if row['pgid'] != pid or pid == self.parent_group:
                    self.reject(role + ' group ownership invalid', row)
                elif not self.rejections and pid in self.first:
                    self.groups.add(pid)
            if pid in self.first and chain and any(
                    p != self.root and p not in self.first for p in chain):
                self.reject('unknown parent in owned chain', row)
            if row['pgid'] in self.groups and (pid not in self.first or not chain) \
                    and not self.terminal_departure(pid, row, by_pid):
                self.reject('foreign or unparented owned-group member', row)
        if len(self.targets) > 1:
            self.reject('more than one target observed')
        return rows

    def terminal_departure(self, pid, row, rows):
        """Prior attach chain + actual native termination; never live authority."""
        if self.role(pid) != 'inferior' or pid in self.tainted:
            return False
        entry = self.native_last.get(pid, {})
        if not entry.get('present') or entry['after']['status'] != 5:
            return False  # absence/no event/ps display do not prove completion
        prior = next((t for t in self.transitions if t.get('pid') == pid
                      and t.get('kind') == 'supported-attach-chain-observation'), None)
        if prior is None and pid in self.acquired_chains:
            chain = self.acquired_chains[pid]
            prior = dict(chain, original_parent=chain['debugger'], tracer_parent=chain['tracer'])
        if prior is None or prior['observation_index'] >= len(self.observations) - 1:
            return False
        original, tracer = prior['original_parent'], prior['tracer_parent']
        return (self.matches(pid, row) and row['ppid'] in (original, tracer)
                and tracer in self.departed and tracer not in self.tainted
                and not self.native_last[tracer]['present'] and tracer not in rows
                and all(p in rows and p not in self.tainted and p not in self.departed
                        and live(self.native_last[p]) for p in (original, self.root))
                and self.matches(original, rows[original])
                and rows[original]['ppid'] == self.root)

    def acknowledge(self, handshake):
        require(not self.rejections and self.last is not None, 'custody rejected')
        pid = handshake['pid']
        require(type(pid) is int and pid > 0 and handshake['pgid'] == pid
                and handshake['run_id'] == self.run_id and handshake['flags'] == 134,
                'invalid inferior handshake')
        require(self.targets == {pid} and pid in self.groups, 'no sole owned target')
        first = self.first[pid]
        current = next((r for r in self.last['rows'] if r['pid'] == pid), None)
        require(current is not None and self.matches(pid, current)
                and live(self.native_last[pid])
                and all(p not in self.departed and p not in self.tainted
                        and live(self.native_last[p]) for p in self.accepted_parents[pid])
                and self.native_last[pid]['after']['path'] == self.binary
                and current['stat'][:1] in STATES - {'Z'},
                'inferior not currently authenticated and live')
        return copy.deepcopy({'schema': SCHEMA, 'run_id': self.run_id,
            'state': 'ACTIVE', 'process': first['row'], 'initial': first,
            'current': current, 'observation': self.last,
            'custody': self.document(),
            'compared_fields': list(COMPARED_FIELDS), 'rejections': []})

    def cleanup_allowed(self, pgid, snapshot):
        """Every current member must still match an exclusive initial identity."""
        self.observe(snapshot)
        if not self.capture_valid:
            return False
        rows = observed_rows(snapshot)
        members = [r for r in rows if r['pgid'] == pgid]
        chains = ancestry(rows, self.root)
        current = {r['pid']: r for r in rows}
        if any(r['pid'] in self.native_last and not live(self.native_last[r['pid']])
               for r in members):
            return False  # expected exit denies signals without inventing a conflict

        def initial_chain(row):
            chain = chains.get(row['pid'])
            return chain is not None and all(
                pid == self.root or pid in self.first and pid not in self.tainted
                and live(self.native_last[pid])
                and self.matches(pid, current[pid])
                and all(p in current and p not in self.departed and p not in self.tainted
                        and live(self.native_last[p]) for p in self.accepted_parents[pid])
                for pid in chain)

        allowed = (type(pgid) is int and pgid in self.groups and pgid != self.parent_group
                   and all(r['pid'] in self.first and r['pid'] not in self.tainted
                           and self.matches(r['pid'], r)
                           and initial_chain(r)
                           for r in members))
        if not allowed:
            self.reject('cannot authenticate group for cleanup: ' + str(pgid))
        return (allowed and bool(members) and self.root not in self.tainted
                and all(live(self.native_last[r['pid']]) and r['stat'][:1] != 'Z'
                        for r in members))


def validate_ack(ack, pid, run_id, binary):
    require(ack['schema'] == SCHEMA and ack['state'] == 'ACTIVE'
            and ack['run_id'] == run_id and ack['rejections'] == [], 'rejected/wrong-run ack')
    require(ack['compared_fields'] == list(COMPARED_FIELDS), 'wrong ack comparison contract')
    first, current = ack['initial']['row'], ack['current']
    require(ack['process'] == first, 'ack initial identity changed')
    require(type(pid) is int and pid > 0 and first['pid'] == first['pgid'] == pid
            and ack['custody']['native_initial'].get(pid,
                ack['custody']['native_initial'].get(str(pid)))['registration']['after']['path']
            == str(binary), 'wrong ack target')
    validate_snapshot(ack['initial']['observation'])
    validate_snapshot(ack['observation'])
    require(first in ack['initial']['observation']['rows']
            and current in ack['observation']['rows']
            and current['stat'][:1] in STATES - {'Z'}, 'ack missing process evidence')
    require(ack['observation']['begin_monotonic'] >=
            ack['initial']['observation']['begin_monotonic'], 'ack time precedes initial')
    chain = ack['initial']['ancestry']
    require(isinstance(chain, list) and len(chain) >= 2 and chain[0] == pid
            and ancestry(ack['initial']['observation']['rows'], chain[-1]).get(pid) == chain
            and pid in ancestry(ack['observation']['rows'], chain[-1]),
            'ack ancestry unproved')
    replayed = replay_document(ack['custody'], binary, run_id)
    expected = replayed.acknowledge({'pid': pid, 'pgid': pid, 'run_id': run_id, 'flags': 134})
    import json
    require(all(json.loads(json.dumps(ack[k])) == json.loads(json.dumps(expected[k]))
                for k in expected), 'ack custody replay mismatch')


def replay_document(document, binary, run_id):
    """Both consumers recompute authorization from all native/ps evidence."""
    import json
    replayed = Custody(document['root_pid'], document['parent_group'], binary, run_id)
    for snapshot in document['observations']:
        replayed.observe(snapshot)
    require(not replayed.rejections, 'custody replay rejected')
    require(json.loads(json.dumps(replayed.document())) == json.loads(json.dumps(document)),
            'custody document differs from replay')
    return replayed


def validate_completion(ack, custody, pid, run_id, binary):
    validate_ack(ack, pid, run_id, binary)
    require(custody['schema'] == SCHEMA and custody['state'] == 'ACTIVE'
            and custody['rejections'] == [] and custody['tainted_pids'] == []
            and custody['run_id'] == run_id, 'custody rejected at completion')
    initial = custody['initial_observations'].get(pid,
              custody['initial_observations'].get(str(pid)))
    require(initial == ack['initial'] and custody['target_ids'] == [pid]
            and pid in custody['owned_groups'] and ack['process'] in custody['processes'],
            'completion lost initial custody')
    require(ack['initial']['ancestry'][-1] == custody['root_pid']
            and pid != custody['parent_group'], 'completion wrong custody owner')
    replay_document(custody, binary, run_id)
    require(custody['observations'][:len(ack['custody']['observations'])] ==
            ack['custody']['observations'], 'completion lost acknowledged history')
