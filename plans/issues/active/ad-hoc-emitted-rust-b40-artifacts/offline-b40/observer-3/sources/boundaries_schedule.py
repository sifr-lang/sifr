"""Fixed six-target schedule; one collector and release/output before next."""
from boundaries_core import full_target_fits, run_id
from coverage_symbols import require


class Schedule:
    def __init__(self, start, collector):
        self.start, self.collector = start, collector
        self.index, self.begun, self.failure = 0, None, None
        self.receipts, self.inventory = [], set()

    def fail(self, reason):
        if self.failure is None:
            self.failure = str(reason)

    def permit(self, now, collector):
        require(self.failure is None and collector is self.collector, 'schedule failed/custody replaced')
        require(self.begun is None and self.index < 6, 'target already active/schedule exhausted')
        full_target_fits(now, self.start)
        require(len(self.receipts) == self.index, 'prior target completion missing')
        self.begun = now
        return {'index': self.index, 'run_id': run_id(self.index),
                'warmup': self.index == 0, 'full_target_seconds': 60}

    def check(self, now, events, evidence_bytes, free_bytes):
        require(self.failure is None, 'sticky schedule failure')
        require(self.begun is not None and 0 <= now - self.begun < 60, 'target including release deadline')
        require(0 <= now - self.start < 510, 'observation window ended')
        require(type(events) is int and 0 <= events <= 256, 'target event cap')
        require(evidence_bytes < 2 * 1024**3 and free_bytes >= 4 * 1024**3, 'storage cap')

    def finish(self, now, release, output, receipt, inventory):
        self.check(now, 0, 0, 4 * 1024**3)
        require(release['state'] == output['state'] == receipt['state'] == 'PASS', 'release/output incomplete')
        require(receipt['index'] == self.index and receipt['run_id'] == run_id(self.index), 'wrong target completion')
        require(self.inventory <= set(inventory), 'cumulative retirement inventory lost')
        self.inventory.update(inventory)
        self.receipts.append(receipt)
        self.begun = None
        self.index += 1
