"""Prepare and consume sysroot graphs before remaining standard-profile work."""
import sys

from .cargo_setup import acquire_cargo_dependencies, enable_offline_cargo, prepare_remaining_graphs
from .cloud_schedule import Schedule
from .paths import REPO_ROOT
from .prepared_sysroot import OWNER_VARIABLE, command
from .profile_commands import run_command
from .resource_admission import ResourceError


def prepare_compact(runner):
    if not sys.platform.startswith('linux'):
        raise ResourceError('compact resource policy requires Linux cgroup v2','unavailable')
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
            status=schedule.step('preparation_'+suite.replace('-','_'),lambda s=suite: run_command(
                [sys.executable,str(REPO_ROOT/'verification/areas/sysroot_release/package_build.py'),
                 '--metadata-suite',s],env=env),allocation='sysroot-metadata',preparation=True,monitor_disk=True)
            if status: return status
    status=schedule.step('area_sysroot_release',lambda:runner.run_area('sysroot_release',selections[0]['suites']),
                         allocation='sysroot-assertions',monitor_disk=True)
    if status: return status
    runner.compact_completed_areas={'sysroot_release'}
    return schedule.step('cargo_cache_setup',lambda:prepare_remaining_graphs(profile,env,schedule.prepare_command,
                         include_sysroot=False),allocation='preparation-coordination',preparation=True,monitor_disk=True)
