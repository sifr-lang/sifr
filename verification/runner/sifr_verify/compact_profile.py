"""Prepare and consume sysroot graphs before remaining standard-profile work."""
import sys

from .cargo_setup import acquire_cargo_dependencies, enable_offline_cargo, prepare_remaining_graphs
from .cloud_schedule import Schedule
from .early_sql import EarlySqlOutcome, run_early_sql
from .paths import REPO_ROOT
from .prepared_sysroot import OWNER_VARIABLE, command
from .profile_commands import run_command
from .resource_admission import ResourceError
from .cargo_resource_forecast import test_cache_hint


def prepare_compact(runner):
    if not sys.platform.startswith('linux'):
        raise ResourceError('compact resource policy requires Linux cgroup v2','unavailable')
    runner.early_sql_outcome=EarlySqlOutcome()
    schedule=Schedule(runner)
    env,profile=runner.env,runner.profile
    runner.compact_schedule=schedule
    print(f'Compact execution journal: {schedule.journal}',flush=True)
    env[OWNER_VARIABLE]=schedule.owner
    env['SIFR_VERIFY_GRAPH_OWNER']=schedule.graph_owner
    env['SIFR_VERIFY_SYSROOT_GRAPH_SESSION']=schedule.owner
    status=schedule.step('preparation_dependencies',lambda: acquire_cargo_dependencies(profile,env,run_command),
                         allocation='dependency-acquisition',preparation=True,monitor_disk=True)
    if status: return status
    enable_offline_cargo(env)
    selections=[area for area in profile['selected_areas'] if area['area']=='sysroot_release']
    if len(selections)!=1:
        raise ResourceError('compact profile requires its canonical sysroot selection','unavailable')
    suites=set(selections[0]['suites'])
    source=bool(suites.intersection({'boundary-equivalence','metadata-structural'}))
    package=bool(suites-{'path-leakage-self-test','metadata-structural'})
    for kind,needed in (('source',source),('package',package)):
        if not needed: continue
        status=schedule.step('preparation_sysroot_'+kind,lambda k=kind: run_command(command('prepare',k),env=env),
                             allocation='sysroot-'+kind,preparation=True,monitor_disk=True)
        if status: return status
    metadata=bool(suites.intersection({'metadata-structural','metadata-corpus'}))
    if metadata:
        # Match each selected library configuration. Do not prepare an unselected
        # corpus solely because the shared package preparer knows about it.
        for suite in sorted(suites.intersection({'metadata-structural','metadata-corpus'})):
            cached=test_cache_hint(REPO_ROOT,env,'sifr_driver',include_library=True)
            schedule.record('metadata-forecast',{'suite':suite,'cache_presence_hint':cached,'assertion_reuse':False})
            status=schedule.step('preparation_'+suite.replace('-','_'),lambda s=suite: run_command(
                [sys.executable,str(REPO_ROOT/'verification/areas/sysroot_release/package_build.py'),
                 '--metadata-suite',s],env=env),allocation='sysroot-metadata-cached' if cached else 'sysroot-metadata',preparation=True,monitor_disk=True)
            if status: return status
    # The structural-only selection executes prepared Rust tests and the
    # development metadata doctor. Installed/corpus/boundary suites retain the
    # larger native-compilation allocation, including unknown future suites.
    allocation='sysroot-structural-assertions' if suites=={'metadata-structural'} else 'sysroot-assertions'
    status=schedule.step('area_sysroot_release',lambda:runner.run_area('sysroot_release',selections[0]['suites']),
                         allocation=allocation,monitor_disk=True)
    if status: return status
    runner.compact_completed_areas={'sysroot_release'}
    runner.early_sql_outcome=run_early_sql(runner,schedule)
    if runner.early_sql_outcome.status and not runner.no_fail_fast:
        return runner.early_sql_outcome.status
    return schedule.step('cargo_cache_setup',lambda:prepare_remaining_graphs(profile,env,schedule.prepare_command,
                         include_sysroot=False,sql_preparation_handled=runner.early_sql_outcome.selected),
                         allocation='preparation-coordination',preparation=True)
