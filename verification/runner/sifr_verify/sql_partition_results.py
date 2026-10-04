"""Fresh SQL invocation parts produce one complete canonical area result."""
from __future__ import annotations

import json
import math
import uuid
from pathlib import Path

from .area_cargo_setup import sql_runner
from .fixture_execution import blocked_case
from .paths import REPO_ROOT
from .profile_results import AreaResultError, validate_area_result


class SqlInvocation:
    def __init__(self, profile_name: str, suites: list[str], *, root: Path = REPO_ROOT):
        self.root = root
        self.adapter = sql_runner()
        self.manifest_bytes = self.adapter.MANIFEST_PATH.read_bytes()
        manifest = json.loads(self.manifest_bytes)
        self.selected = self.adapter.select_suites(manifest, set(suites))
        self.final = root / 'target/verification/areas' / f'sql-platform-{profile_name}-results.json'
        self.final.unlink(missing_ok=True)
        self.slug = f'sql-platform-parts/{uuid.uuid4()}'
        self.profile_name = profile_name
        self.results = {}
        self.reasons = {}
        self.claimed = set()
        self.invalid = False

    def part_slug(self, label: str) -> str:
        if label not in {'build', 'remaining'}:
            raise AreaResultError('unknown SQL invocation part')
        return f'{self.slug}/{label}'

    def part_path(self, label: str) -> Path:
        return self.root / 'target/verification/areas' / f'{self.part_slug(label)}-{self.profile_name}-results.json'

    def accept(self, label: str, names: list[str], status: int, *, failure_reason: str | None = None) -> int:
        expected = [suite for suite in self.selected if suite['name'] in names]
        try:
            if (not names or len(set(names)) != len(names) or len(expected) != len(names)
                    or self.claimed.intersection(names)):
                raise ValueError('duplicate or unknown SQL part selection')
            self.claimed.update(names)
            payload = json.loads(self.part_path(label).read_text())
            if (not isinstance(payload, dict) or type(payload.get('schema_version')) is not int or payload.get('schema_version') != 1 or payload.get('area') != 'sql_platform'
                    or payload.get('bless') is not False
                    or payload.get('manifest') != 'verification/areas/sql_platform/manifest.json'):
                raise ValueError('invalid SQL part identity')
            rows = payload['suites']
            if not isinstance(rows, list) or len(rows) != len(expected):
                raise ValueError('incomplete SQL part suites')
            for row, suite in zip(rows, expected, strict=True):
                self.validate_suite(row, suite)
            variants = sum(row['total_variants'] for row in rows)
            failures = sum(row['total_failures'] for row in rows)
            if (not isinstance(payload['summary'], dict)
                    or any(type(value) is not int for value in payload['summary'].values())
                    or payload['summary'] != dict(total_variants=variants, total_failures=failures,
                                          blocking_failures=failures, non_blocking_failures=0)):
                raise ValueError('SQL part summary differs from actual cases')
            if bool(status) != bool(failures):
                raise ValueError('SQL part process outcome disagrees with its result')
            self.results.update((row['name'], row) for row in rows)
        except (OSError, ValueError, KeyError, TypeError) as error:
            for name in names:
                self.results.pop(name, None)
            self.invalid = self.invalid or not set(names) <= {suite['name'] for suite in self.selected}
            reason = f'{label}: {failure_reason or f"process status {status}"}; invalid or missing result: {error}'
            self.reasons.update((name, reason) for name in names)
            return status or 2
        return status

    def validate_suite(self, row, suite):
        if (not isinstance(row, dict) or row.get('name') != suite['name']
                or row.get('owner') != 'compiler/sql-platform' or row.get('blocking') is not True
                or row.get('runner') != 'sql_platform'):
            raise ValueError('SQL suite identity mismatch')
        cases = row['cases']
        if not isinstance(cases, list) or len(cases) != len(suite['cases']):
            raise ValueError('SQL case coverage mismatch')
        failures = 0
        for actual, expected in zip(cases, suite['cases'], strict=True):
            if not isinstance(actual, dict) or any(actual.get(key) != expected[key] for key in ('id', 'entry', 'command')):
                raise ValueError('SQL case identity/order mismatch')
            variants = actual['variants']
            if not isinstance(variants, list) or len(variants) != 1:
                raise ValueError('SQL variant count mismatch')
            variant = variants[0]
            if not isinstance(variant, dict):
                raise ValueError('invalid SQL variant')
            command = expected['command']
            code = variant.get('actual_exit_code')
            passed = code == expected['expect_exit_code']
            duration = variant.get('duration_ms')
            if (variant.get('label') != command or variant.get('argv') != self.adapter.COMMANDS[command]
                    or type(code) is not int or type(variant.get('expected_exit_code')) is not int
                    or variant.get('expected_exit_code') != expected['expect_exit_code']
                    or variant.get('status') != ('pass' if passed else 'fail')
                    or variant.get('mismatches') != ([] if passed else ['unexpected-exit'])
                    or type(duration) not in (int, float) or not math.isfinite(duration) or duration < 0):
                raise ValueError('SQL variant execution evidence mismatch')
            failures += not passed
        if (type(row.get('total_variants')) is not int or row['total_variants'] != len(cases)
                or type(row.get('total_failures')) is not int or row['total_failures'] != failures
                or type(row.get('failed_cases')) is not int or row['failed_cases'] != failures):
            raise ValueError('SQL suite counters mismatch')

    def finish(self, *, missing_reason: str) -> dict:
        if self.adapter.MANIFEST_PATH.read_bytes() != self.manifest_bytes:
            raise AreaResultError('SQL manifest changed during this invocation')
        rows = []
        for suite in self.selected:
            row = None if self.invalid else self.results.get(suite['name'])
            if row is None:
                reason = self.reasons.get(suite['name'], missing_reason)
                cases = [blocked_case(case, reason)[0] for case in suite['cases']]
                row = dict(name=suite['name'], owner='compiler/sql-platform', blocking=True,
                           runner='sql_platform', cases=cases, failed_cases=len(cases),
                           total_variants=len(cases), total_failures=len(cases))
            rows.append(row)
        failures = sum(row['total_failures'] for row in rows)
        payload = dict(schema_version=1, area='sql_platform', bless=False,
                       manifest='verification/areas/sql_platform/manifest.json', suites=rows,
                       summary=dict(total_variants=sum(row['total_variants'] for row in rows),
                                    total_failures=failures, blocking_failures=failures, non_blocking_failures=0))
        self.final.parent.mkdir(parents=True, exist_ok=True)
        temporary = self.final.with_suffix('.json.tmp')
        temporary.write_text(json.dumps(payload, indent=2, sort_keys=True))
        temporary.replace(self.final)
        if not failures:
            validate_area_result(self.final, area='sql_platform', expected_suites=[row['name'] for row in rows])
        return payload
