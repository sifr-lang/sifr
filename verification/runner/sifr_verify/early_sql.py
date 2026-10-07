"""Qualify the clean SQL graph before retaining unrelated SQL test graphs."""
from __future__ import annotations

from dataclasses import dataclass
import math
import time

from .area_cargo_setup import sql_preparation_commands
from .assertion_resource_forecast import SQL_BUILD_ALLOCATION, assertion_allocation
from .fixture_inventory import inventory
from .errors import VerificationError
from .profile_commands import CommandFailed
from .resource_admission import ResourceError
from .sql_partition_results import SqlInvocation

DURATION = 'SIFR_VERIFY_SAFETY_DEADLINE_SECONDS'
STEP_DURATION = 'SIFR_VERIFY_STEP_SAFETY_DEADLINE_SECONDS'


@dataclass(frozen=True)
class EarlySqlOutcome:
    # Handled in this invocation includes failed and blocked outcomes; it never
    # claims a prior passing assertion or authorizes cross-run reuse.
    selected: bool = False
    preparation_status: int = 0
    assertion_status: int | None = None

    @property
    def status(self) -> int:
        return self.assertion_status or self.preparation_status or 0


class AssertionBudget:
    """One original assertion duration, excluding interleaved preparations."""
    def __init__(self, runner, schedule):
        self.runner = runner
        self.limit = float(schedule.policy['assertion_command_deadline_seconds'])
        for variable in (DURATION, STEP_DURATION):
            if variable in runner.env:
                inherited = float(runner.env[variable])
                if not math.isfinite(inherited) or inherited <= 0:
                    raise ResourceError('inherited safety duration must be positive and finite', 'unavailable')
                self.limit = min(self.limit, inherited)
        self.spent = 0.0
        self.failure_detail = None

    def run(self, schedule, name, callback, *, allocation):
        self.failure_detail = None
        previous = {key:self.runner.env.get(key) for key in (DURATION, STEP_DURATION)}
        remaining = self.limit - self.spent
        # Keep a positive duration for scheduler admission; exhaustion is an
        # explicit timeout before any child spawn, never a renewed allowance.
        self.runner.env[DURATION] = str(max(remaining, 1e-9))
        if previous[STEP_DURATION] is not None:
            # run_command prefers STEP over the ordinary command duration.
            self.runner.env[STEP_DURATION] = self.runner.env[DURATION]
        def timed():
            if remaining <= 0:
                raise CommandFailed(124, 'timeout')
            started = time.monotonic()
            try:
                callback()
            finally:
                self.spent += time.monotonic() - started
        try:
            # Existing process execution also clamps to inherited absolute
            # deadlines. Never remove or extend that deadline across preparation.
            return schedule.step(name, timed, allocation=allocation, monitor_disk=True, propagate_failure=True)
        except (CommandFailed, VerificationError, OSError) as error:
            kind = getattr(error, 'classification', None) or getattr(error, 'cause', None) or type(error).__name__
            self.failure_detail = f'{kind}: {error}'
            return error.returncode if isinstance(error, CommandFailed) else 2
        finally:
            for key, value in previous.items():
                if value is None:
                    self.runner.env.pop(key, None)
                else:
                    self.runner.env[key] = value


def run_early_sql(runner, schedule) -> EarlySqlOutcome:
    if assertion_allocation('area_sql_platform', runner.profile) != SQL_BUILD_ALLOCATION:
        return EarlySqlOutcome()
    selection = next(area for area in runner.profile['selected_areas'] if area['area'] == 'sql_platform')
    suites = selection['suites']
    remaining = [suite for suite in suites if suite != 'build-qualification']
    results = SqlInvocation(runner.profile_name, suites, root=schedule.root)
    budget = AssertionBudget(runner, schedule)

    def part(label, selected, allocation):
        status = budget.run(schedule, 'sql_' + label + '_assertions',
            lambda: runner.run_area('sql_platform', selected, result_slug=results.part_slug(label)),
            allocation=allocation)
        return results.accept(label, selected, status, failure_reason=budget.failure_detail)

    failed = part('build', ['build-qualification'], SQL_BUILD_ALLOCATION)
    prepared = 0
    # Legacy SQL executes every case even after an ordinary assertion failure.
    # Keep that behavior for a genuine completed build result; outer fail-fast
    # takes effect only after the canonical whole-area outcome. Infrastructure
    # without complete execution evidence may stop dependent work earlier.
    build_executed = 'build-qualification' in results.results
    if remaining and (not failed or build_executed or runner.no_fail_fast):
        def prepare():
            for command in sql_preparation_commands(remaining):
                schedule.prepare_command(command, env=runner.env)
        prepared = schedule.step('preparation_sql_platform', prepare,
                                 allocation='preparation-coordination', preparation=True)
        failed = failed or prepared
        if not prepared:
            status = part('remaining', remaining, 'remaining-assertions')
            failed = failed or status
        else:
            results.reasons.update((name, 'preparation_sql_platform failed; assertions not executed')
                                   for name in remaining)

    def aggregate():
        if inventory(schedule.root) != schedule.key['inputs']['source']:
            raise ResourceError('validation inputs changed before SQL aggregation', 'unavailable')
        payload = results.finish(missing_reason='SQL prerequisite or fail-fast prevented execution')
        if failed or payload['summary']['blocking_failures']:
            raise CommandFailed(failed or 2)
    status = runner.execute_step('area_sql_platform', aggregate, prior_elapsed_ms=int(budget.spent * 1000))
    return EarlySqlOutcome(True, prepared, status)
