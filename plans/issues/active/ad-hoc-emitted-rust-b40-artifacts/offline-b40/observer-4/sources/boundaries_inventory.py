"""Cumulative retirement evidence, separate from custody and signal authority."""
import copy
import json
from pathlib import Path
from coverage_custody import ancestry


class Inventory:
    def __init__(self, root, binary=None):
        self.root, self.binary = root, str(binary) if binary is not None else None
        self.pids, self.groups, self.rooted = {root}, set(), {root}
        self.receipts, self.candidates, self.launch_records = [], [], []
        self.errors = []

    def candidate(self, sample, source, rooted=False):
        if not isinstance(sample, dict):
            return
        pid, group = sample.get('pid'), sample.get('pgid')
        if type(pid) is not int or pid <= 0:
            return
        self.pids.add(pid)
        if rooted:
            self.rooted.add(pid)
        if type(group) is not int or group <= 0:
            return
        required = pid == group and pid != self.root and (
            pid in self.rooted or self.binary is not None and sample.get('path') == self.binary)
        record = {'pid': pid, 'pgid': group, 'path': sample.get('path'),
                  'source': source, 'required_absence': required,
                  'authority': 'retirement only; never signals', 'sample': copy.deepcopy(sample)}
        if record not in self.candidates:
            self.candidates.append(record)
        if required:
            self.groups.add(group)

    def collect(self, snapshot, source='capture'):
        """Preserve original/nested ps and every partial native sample before throw."""
        if not isinstance(snapshot, dict):
            return
        rows = snapshot.get('rows', [])
        if isinstance(rows, list):
            safe = [r for r in rows if isinstance(r, dict)
                    and all(type(r.get(k)) is int for k in ('pid', 'ppid', 'pgid'))]
            chains = ancestry(safe, self.root)
            self.rooted.update(chains)
            selected = [r for r in safe if r['pid'] in chains
                        or r['pid'] in self.pids or r['pgid'] in self.groups]
            for row in selected:
                self.candidate(row, source + '.rows', row['pid'] in chains)
            probe = snapshot.get('probe_pid')
            if type(probe) is int and probe > 0:
                self.pids.add(probe)
            if selected or probe:
                self.receipts.append({'begin': snapshot.get('begin_monotonic'),
                    'end': snapshot.get('end_monotonic'), 'probe_pid': probe,
                    'rows': copy.deepcopy(selected), 'error': snapshot.get('error')})
        native = snapshot.get('native')
        if isinstance(native, dict):
            self.pids.update(int(p) for p in native.get('records', {}) if str(p).isdigit())
        # Called recursively only on evidence structures (never an arbitrary
        # host table row); ps selects rooted/known candidates above.
        if 'rows' not in snapshot:
            self.candidate(snapshot, source)
        for key, value in snapshot.items():
            if key in ('rows', 'raw'):
                continue
            if isinstance(value, dict):
                self.collect(value, source + '.' + key)
            elif isinstance(value, list):
                for index, child in enumerate(value):
                    if isinstance(child, dict):
                        self.collect(child, source + '.' + key + '[' + str(index) + ']')

    def collect_launch_records(self, out, run_id):
        """Only the six exact owned paths; called before inspect and in finalizers."""
        for index in range(6):
            directory = Path(out) / ('target-' + str(index))
            for name in ('inferior.json', 'observer-result.json'):
                path = directory / name
                if not path.exists():
                    continue
                try:
                    record = json.loads(path.read_text())
                    value = {'path': str(path), 'record': copy.deepcopy(record)}
                    if value not in self.launch_records:
                        self.launch_records.append(value)
                    pid = record.get('pid')
                    exact = record.get('run_id') == run_id(index)
                    permit_path = directory / 'launch-permit.json'
                    permit = json.loads(permit_path.read_text()) if permit_path.exists() else {}
                    exact = exact and permit.get('run_id') == run_id(index) and permit.get('index') == index
                    if (exact and name == 'observer-result.json' and record.get('target_launches') == 0
                            and (pid is None or pid == 0)):
                        continue  # Recorded prelaunch terminal: no target identity to retire.
                    if exact and type(pid) is int and pid > 0:
                        # Exact owned observer PID is itself a target-group
                        # obligation even if its partial terminal omits PGID.
                        self.candidate(dict(record, pid=pid, pgid=record.get('pgid', pid)),
                                       str(path), rooted=True)
                        self.candidate({'pid': pid, 'pgid': pid}, str(path) + '.target-own-group', rooted=True)
                    else:
                        self.candidate(record, str(path) + '.unproved')
                        error = 'unproved owned launch record: ' + str(path)
                        if error not in self.errors:
                            self.errors.append(error)
                except (ValueError, OSError, TypeError) as exc:
                    error = str(path) + ': ' + repr(exc)
                    if error not in self.errors:
                        self.errors.append(error)

    def load(self, document):
        """Import cumulative producer obligations without turning them into authority."""
        if document.get('root') != self.root:
            raise ValueError('retirement inventory root differs')
        self.pids.update(document['pids'])
        self.groups.update(document['groups'])
        self.rooted.update(document.get('rooted', []))
        self.receipts.extend(copy.deepcopy(document.get('receipts', [])))
        self.launch_records.extend(copy.deepcopy(document.get('launch_records', [])))
        self.errors.extend(document.get('errors', []))
        for record in document.get('candidates', []):
            self.candidates.append(copy.deepcopy(record))
            if record['required_absence']:
                self.groups.add(record['pgid'])

    def document(self):
        return {'authority': 'read-only absence obligation; never signal authority',
                'root': self.root, 'binary': self.binary, 'pids': sorted(self.pids),
                'rooted': sorted(self.rooted), 'groups': sorted(self.groups),
                'receipts': copy.deepcopy(self.receipts), 'candidates': copy.deepcopy(self.candidates),
                'launch_records': copy.deepcopy(self.launch_records), 'errors': list(self.errors)}


def validate_absence(receipt, inventory, keepers=()):
    required_groups = {str(g) for g in inventory.groups - set(keepers)}
    required_pids = inventory.pids - set(keepers)
    if (receipt.get('state') != 'PASS' or inventory.errors
            or set(receipt.get('groups', {})) != required_groups
            or any(value is not False for value in receipt['groups'].values())
            or not required_pids <= set(receipt.get('pids', []))
            or any(str(p) not in receipt.get('after', {}) or receipt['after'][str(p)] is not None
                   for p in required_pids)):
        raise ValueError('complete retirement PID/group coverage unproved')
