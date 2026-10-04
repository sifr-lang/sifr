"""Execute the whole selected SQL area before unrelated retained preparations."""
from __future__ import annotations

from dataclasses import dataclass

from .area_cargo_setup import sql_preparation_commands
from .assertion_resource_forecast import SQL_BUILD_ALLOCATION, assertion_allocation


@dataclass(frozen=True)
class EarlySqlOutcome:
    # Selected means handled in this invocation, including a blocked or failed
    # area. It never claims a passing assertion or authorizes cross-run reuse.
    selected: bool = False
    preparation_status: int = 0
    assertion_status: int | None = None

    @property
    def status(self) -> int:
        return self.preparation_status or self.assertion_status or 0


def run_early_sql(runner, schedule) -> EarlySqlOutcome:
    if assertion_allocation('area_sql_platform', runner.profile) != SQL_BUILD_ALLOCATION:
        return EarlySqlOutcome()
    selection = next(area for area in runner.profile['selected_areas']
                     if area['area'] == 'sql_platform')

    def prepare():
        for command in sql_preparation_commands(selection['suites']):
            schedule.prepare_command(command, env=runner.env)

    # Coordination must not impose a cumulative floor on independently
    # admitted preparations; every original Cargo command still executes.
    prepared = schedule.step('preparation_sql_platform', prepare,
                             allocation='preparation-coordination', preparation=True)
    if prepared:
        runner.block_step('area_sql_platform', 'preparation_sql_platform')
        return EarlySqlOutcome(True, prepared)
    status = schedule.step('area_sql_platform',
        lambda: runner.run_area('sql_platform', selection['suites']),
        allocation=SQL_BUILD_ALLOCATION, monitor_disk=True)
    return EarlySqlOutcome(True, 0, status)
