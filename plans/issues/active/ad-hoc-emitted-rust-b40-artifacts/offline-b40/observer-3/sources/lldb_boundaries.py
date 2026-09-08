"""B24 one-session entry/return observer over qualified B38 entry binding."""
import json
import os
import signal
import sys
import time
import traceback
from pathlib import Path
import lldb
from coverage_output import configure_launch
from boundaries_core import Pairing, Counters, RETURN_FAMILIES, run_id, preflight_count
from coverage_custody import validate_ack
from coverage_address import initial_entry_matches
from coverage_module import Resolver
from coverage_symbols import (EARLY, FAMILIES, TRAPS, Protocol, bind, check_site,
                              expected_path, load_contract, require, resolve_identity,
                              stop_evidence)

E = Path('/private/tmp/sifr-b40.urwpgv/evidence')
OUT = E / 'boundaries'
ROOT = E.parent / 'sifr'
BINARY = E / 'sifr-experiment09'


def save(name, value):
    path = OUT / name
    temporary = path.with_suffix('.tmp')
    temporary.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')
    temporary.replace(path)


def observe(debugger, index):
    global OUT
    OUT = E / 'boundaries' / ('target-' + str(index))
    RUN_ID = run_id(index)
    begun = time.monotonic()
    launch_start = None
    process = None
    resolver = None
    target = None
    events = []
    arming = []
    identities = []
    sites = {}
    later_ids = {}
    protocol = Protocol()
    pairing = Pairing()
    counters = Counters()
    return_sites = {}
    entries = load_contract()
    stage = 'setup'
    output = {'state': 'INCONCLUSIVE', 'target_launches': 0, 'run_id': RUN_ID,
              'python': sys.version, 'clock': 'embedded local elapsed only'}

    def check():
        require(not (E / 'boundaries/stop.json').exists(), 'outer custodian requested stop')
        start = launch_start if launch_start is not None else begun
        limit = 60 if launch_start is not None else 30
        require(time.monotonic() - start < limit, 'embedded elapsed deadline')

    def module_row(module):
        header = module.GetObjectFileHeaderAddress()
        return {'path': str(module.GetFileSpec()), 'uuid': module.GetUUIDString(),
                'triple': module.GetTriple(), 'header_file': header.GetFileAddress(),
                'header_load': header.GetLoadAddress(target)}

    def modules():
        result = []
        for module in target.modules:
            row = module_row(module)
            row['sections'] = [{'name': s.GetName(), 'file': s.GetFileAddress(),
                                'load': s.GetLoadAddress(target), 'bytes': s.GetByteSize()}
                               for s in module.sections]
            result.append(row)
        return result

    def get_resolver():
        nonlocal resolver
        if resolver is None:
            resolver = Resolver(lldb, target, process, entries, lambda: protocol.epoch,
                                lambda rows: save('module-instance-resolutions.json', rows))
        return resolver

    def current_module(label, required=True):
        return get_resolver().current_module(label, required)

    def identity(label, module):
        return get_resolver().identity(label, module)

    def resolve(label):
        check()
        return get_resolver().resolve(label)

    def site_row(bp):
        location = bp.GetLocationAtIndex(0)
        return {'epoch': protocol.epoch, 'valid': bp.IsValid(),
                'enabled': bp.IsEnabled() and location.IsEnabled(),
                'resolved': location.IsResolved(), 'locations': bp.GetNumLocations(),
                'load': location.GetLoadAddress(), 'id': bp.GetID(),
                'location_id': location.GetID()}

    def arm(label, bp=None):
        binding = resolve(label)
        if bp is None:
            bp = target.BreakpointCreateByAddress(binding['load'])
        site = site_row(bp)
        arming.append({'label': label, 'binding': binding, 'site': site,
                       'stop_id': process.GetStopID()})
        save('site-arming.json', arming)
        check_site(binding, site, protocol.epoch)
        sites[label] = {'binding': binding, 'site': site}

    def arm_dyld():
        module = current_module('prepare')
        # Account for the whole registered dyld identity set, including the
        # non-trapped helpers that define the static handover chain.
        for label in ('initial_entry', 'notifier', 'handover_prepare', 'handover_complete',
                      'cache_restart', 'prepare', 'fixups', 'generic'):
            row, candidates, unused = identity(label, module)
            identities.append({'label': label, 'module': row, 'symbol': candidates[0],
                               'epoch': protocol.epoch, 'stop_id': process.GetStopID()})
        save('identity-resolutions.json', identities)
        for label in ('prepare', 'fixups', 'generic', 'notifier'):
            arm(label)

    def refresh_later():
        for label in sorted(FAMILIES - EARLY):
            module = current_module(label, required=False)
            if module is None:
                require(label not in sites, ('required image unloaded', label))
                require(label not in ('constructor', 'main', 'completion'), 'app image missing')
                continue
            bp = target.FindBreakpointByID(later_ids[label])
            require(bp.IsValid(), ('lost pending breakpoint', label))
            if label not in sites:
                arm(label, bp)

    def verify_sites():
        for label, value in sites.items():
            binding = resolve(label)
            old = value['binding']
            require(all(binding[k] == old[k] for k in ('epoch', 'load', 'module', 'section_load')),
                    ('site binding changed without handover', label))
            site = site_row(target.FindBreakpointByID(value['site']['id']))
            check_site(binding, site, protocol.epoch)
            require(site['location_id'] == value['site']['location_id'], ('location identity changed', label))

    def clear_epoch():
        for label in sorted(TRAPS):
            require(target.BreakpointDelete(sites[label]['site']['id']), ('cannot delete old dyld site', label))
        # Named later breakpoints remain pending across the LLDB module removal.
        # Their bindings, including app ones, must be rebuilt in the new epoch.
        sites.clear()

    def thread_row(thread):
        frame = thread.GetFrameAtIndex(0)
        pc_register = frame.FindRegister('pc')
        pc_read_error = lldb.SBError()
        pc_value = pc_register.GetValueAsUnsigned(pc_read_error, lldb.LLDB_INVALID_ADDRESS)
        pc_error = pc_register.GetError()
        return {'tid': thread.GetThreadID(), 'reason': thread.GetStopReason(),
                'description': thread.GetStopDescription(1024),
                'data': [thread.GetStopReasonDataAtIndex(i) for i in range(thread.GetStopReasonDataCount())],
                'pc': frame.GetPC(), 'function': frame.GetFunctionName(),
                'pc_register': {'valid': pc_register.IsValid(), 'value': pc_value,
                                'error_success': pc_error.Success(), 'error': str(pc_error),
                                'read_success': pc_read_error.Success(), 'read_error': str(pc_read_error),
                                'diagnostic_only': True},
                'module': str(frame.GetModule().GetFileSpec()),
                'x0': frame.FindRegister('x0').GetValueAsUnsigned(lldb.LLDB_INVALID_ADDRESS),
                'x1': frame.FindRegister('x1').GetValueAsUnsigned(lldb.LLDB_INVALID_ADDRESS)}

    def return_site(thread, call):
        key = (thread.GetThreadID(), call['caller']['pc'])
        if key not in return_sites:
            bp = target.BreakpointCreateByAddress(key[1])
            bp.SetThreadID(key[0])
            require(bp.GetThreadID() == key[0], 'return thread constraint missing')
            row = site_row(bp)
            require(row['valid'] and row['enabled'] and row['resolved']
                    and row['locations'] == 1 and row['load'] == key[1], 'ambiguous return site')
            return_sites[key] = row
        return return_sites[key]

    def caller_row(thread):
        require(thread.GetNumFrames() > 1, 'missing unwound caller frame')
        caller = thread.GetFrameAtIndex(1)
        require(caller.IsValid(), 'invalid unwound caller frame')
        address = target.ResolveLoadAddress(caller.GetPC())
        module = caller.GetModule()
        region = lldb.SBMemoryRegionInfo()
        ok = process.GetMemoryRegionInfo(caller.GetPC(), region).Success()
        require(address.IsValid() and address.GetModule() == module,
                'unwound caller address/module identity disagrees')
        return {'pc': caller.GetPC(), 'cfa': caller.GetCFA(), 'valid': caller.IsValid(),
                'module': str(module.GetFileSpec()), 'uuid': module.GetUUIDString(),
                'executable': ok and region.IsExecutable()
                    and region.GetRegionBase() <= caller.GetPC() < region.GetRegionEnd()}

    try:
        require(not (OUT / 'observer-result.json').exists(), 'no repeat observer')
        save('counter-layout.json', counters.layout)
        target = debugger.CreateTarget(str(BINARY))
        require(target.IsValid(), 'invalid target')
        pending = []
        for label in sorted(FAMILIES - EARLY):
            entry = entries[label]
            bp = target.BreakpointCreateByName(entry['exact_lookup'], Path(expected_path(entry)).name)
            require(bp.IsValid(), ('invalid pending breakpoint', label))
            preflight_count(label, bp.GetNumLocations())
            later_ids[label] = bp.GetID()
            pending.append({'label': label, 'id': bp.GetID(), 'exact_lookup': entry['exact_lookup'],
                            'locations': bp.GetNumLocations(), 'installed_claim': False})
        save('pending-sites.json', pending)
        check()
        save('launch-intent.json', {'debugger_pid': os.getpid(), 'flags': 134,
             'elapsed_before_launch': time.monotonic() - begun, 'target_limit_seconds': 60})
        launch = lldb.SBLaunchInfo(['fmt', '--check', '--no-cache',
                                 str(ROOT / 'verification/areas/performance/formatter_project')])
        launch.SetWorkingDirectory(str(ROOT))
        launch.SetLaunchFlags(lldb.eLaunchFlagDebug | lldb.eLaunchFlagStopAtEntry |
                              lldb.eLaunchFlagLaunchInSeparateProcessGroup)
        require(launch.GetLaunchFlags() == 134, 'wrong launch flags')
        route = json.loads((OUT / 'output-route.json').read_text())
        output['output_actions'] = configure_launch(launch, route, OUT, RUN_ID, BINARY)
        stage = 'launch'
        launch_start = time.monotonic()
        error = lldb.SBError()
        process = target.Launch(launch, error)
        require(process.IsValid(), 'invalid process')
        pid = process.GetProcessID()
        output['pid'] = pid
        output['target_launches'] = int(pid not in (0, lldb.LLDB_INVALID_PROCESS_ID))
        require(output['target_launches'] == 1, 'invalid process ID')
        save('inferior.json', {'pid': pid, 'pgid': os.getpgid(pid), 'flags': launch.GetLaunchFlags(),
                               'launch_error': str(error), 'run_id': RUN_ID})
        require(error.Success(), str(error))
        require(os.getpgid(pid) == pid and pid != os.getpgrp(), 'invalid target group')
        while not (OUT / 'custody-ack.json').exists():
            check()
            time.sleep(0.01)
        ack = json.loads((OUT / 'custody-ack.json').read_text())
        validate_ack(ack, pid, RUN_ID, BINARY)
        while True:
            check()
            state = process.GetState()
            if state == lldb.eStateExited:
                output['exit'] = process.GetExitStatus()
                output['exited'] = True
                break
            require(state not in (lldb.eStateInvalid, lldb.eStateDetached, lldb.eStateCrashed), ('unexpected state', state))
            if state != lldb.eStateStopped or process.GetStopID() == protocol.last_stop:
                time.sleep(0.005)
                continue
            protocol.observe_stop(process.GetStopID())
            require(len(events) < 256, 'event ceiling reached')
            threads = [t for t in process if t.GetStopReason() not in (lldb.eStopReasonNone, lldb.eStopReasonInvalid)]
            event = {'sequence': len(events), 'stop_id': process.GetStopID(), 'epoch': protocol.epoch,
                     'elapsed': time.monotonic() - launch_start, 'threads': [thread_row(t) for t in threads],
                     'modules': modules(), 'sites': dict(sites)}
            event['counter'] = counters.sample(pid)
            events.append(event)
            save('events.json', events)
            counters.accept(event['counter'])
            require(len(threads) == 1, 'ambiguous stopped threads')
            thread = threads[0]
            frame = thread.GetFrameAtIndex(0)
            raw = event['threads'][0]
            is_bp = thread.GetStopReason() == lldb.eStopReasonBreakpoint
            evidence = stop_evidence(event)
            pairs = evidence['pairs']
            matched_returns = [key for key, site in return_sites.items()
                               if (site['id'], site['location_id']) in pairs]
            if protocol.phase == 'initial':
                stage = 'initial-entry'
                require(thread.GetStopReason() in (lldb.eStopReasonSignal, lldb.eStopReasonExec), 'not initial exec/signal stop')
                if thread.GetStopReason() == lldb.eStopReasonSignal:
                    require(thread.GetStopReasonDataAtIndex(0) in (signal.SIGSTOP, signal.SIGTRAP), 'unexpected initial signal')
                module = current_module('initial_entry')
                address = target.ResolveLoadAddress(raw['pc'])
                get_resolver().initial_address(module, address)
                region = lldb.SBMemoryRegionInfo()
                require(process.GetMemoryRegionInfo(raw['pc'], region).Success(), 'initial region query failed')
                header = module.GetObjectFileHeaderAddress()
                section = module.FindSection('__TEXT')
                require(section.IsValid(), 'missing initial TEXT')
                coordinates = {'module_path': str(module.GetFileSpec()), 'module_uuid': module.GetUUIDString(),
                    'pc_module_path': str(address.GetModule().GetFileSpec()), 'pc_module_uuid': address.GetModule().GetUUIDString(),
                    'pc': raw['pc'], 'pc_file': address.GetFileAddress(), 'header_file': header.GetFileAddress(),
                    'header_load': header.GetLoadAddress(target), 'section_name': section.GetName(),
                    'section_file': section.GetFileAddress(), 'section_load': section.GetLoadAddress(target),
                    'section_bytes': section.GetByteSize(), 'region_start': region.GetRegionBase(),
                    'region_end': region.GetRegionEnd(), 'executable': region.IsExecutable()}
                event['initial_coordinates'] = coordinates
                event['initial_coordinate_match'] = initial_entry_matches(coordinates)
                save('events.json', events)
                require(event['initial_coordinate_match'], 'initial coordinate predicate rejected')
                stage = 'initial-trap-arming'
                arm_dyld()
                refresh_later()
                protocol.initial(sites)
            elif protocol.phase == 'handover':
                stage = 'handover-mode0-arming'
                require(is_bp and raw['x0'] == 0 and evidence['current_map'], 'missing mapped mode0 handover stop')
                notifier = resolve('notifier')
                require(raw['pc'] == notifier['load'], 'handover stopped outside new notifier')
                arm_dyld()
                refresh_later()
                protocol.handover_in(evidence, sites)
                event['kind'] = 'handover-mode0'
            elif matched_returns:
                stage = 'return-classification'
                require(len(matched_returns) == 1, 'ambiguous return breakpoint')
                require(not any((v['site']['id'], v['site']['location_id']) in pairs
                                for v in sites.values()), 'competing entry/return breakpoint')
                key = matched_returns[0]
                require(key == (thread.GetThreadID(), frame.GetPC()), 'wrong return thread/PC')
                bp = target.FindBreakpointByID(return_sites[key]['id'])
                require(site_row(bp) == return_sites[key] and bp.GetThreadID() == key[0], 'return site identity changed')
                completed = pairing.leave(event['sequence'], thread.GetThreadID(), frame.GetPC(), frame.GetCFA(),
                                          str(frame.GetModule().GetFileSpec()), frame.GetModule().GetUUIDString())
                event.update(kind='return', label=completed['label'], call=completed)
                if not any(c['caller']['pc'] == key[1] for c in pairing.stacks.get(key[0], [])):
                    require(target.BreakpointDelete(return_sites.pop(key)['id']), 'return deletion failed')
            elif protocol.is_notifier(evidence):
                stage = 'notifier-classification'
                if evidence['current_map']:
                    verify_sites()
                event['kind'] = protocol.notifier(evidence)
                event['notifier_mode'] = raw['x0']
                if thread.GetNumFrames() > 1:
                    caller = thread.GetFrameAtIndex(1)
                    event['caller'] = {'pc': caller.GetPC(), 'function': caller.GetFunctionName(),
                                       'module': str(caller.GetModule().GetFileSpec())}
                if protocol.phase == 'handover':
                    clear_epoch()
                else:
                    refresh_later()
                    protocol.install(sites)
            else:
                stage = 'entry-classification'
                require(is_bp and len(raw['data']) >= 2 and len(raw['data']) % 2 == 0, 'unexpected stop or invalid breakpoint data')
                refresh_later()
                labels = {label for label, value in sites.items()
                          if (value['site']['id'], value['site']['location_id']) in pairs}
                require(len(labels) == 1, ('unknown/ambiguous entry site', pairs))
                label = labels.pop()
                stage = 'entry-' + label
                verify_sites()
                protocol.entry(label, evidence, sites[label])
                event['entry_label'] = label
                event.update(kind='entry', label=label)
                if label in RETURN_FAMILIES:
                    call = pairing.enter(label, event['sequence'], thread.GetThreadID(),
                                         frame.GetPC(), frame.GetCFA(), caller_row(thread))
                    event['call'] = dict(call)
                    event['return_site'] = dict(return_site(thread, call))
            if protocol.phase != 'handover':
                verify_sites()
            event['sites_before_continue'] = dict(sites)
            event['phase_before_continue'] = protocol.phase
            event['continue'] = True
            save('events.json', events)
            check()
            require(process.Continue().Success(), 'continue failed')
        stage = 'completion'
        require(output['exit'] == 0, 'inferior did not exit successfully')
        output['intervals'] = pairing.finish()
        require(not return_sites, 'live return sites remain')
        protocol.finish(output['exit'], 2)  # Real two-file verification is the outer canonical consumer.
        protocol.phase = 'active'
        output['state'] = 'ENTRIES_COMPLETE'
    except BaseException:
        output['error'] = traceback.format_exc()
    finally:
        if process is not None and process.IsValid() and process.GetState() != lldb.eStateExited:
            output['kill_result'] = str(process.Kill())
        output.update(stage=stage, hits=protocol.hits, event_count=len(events),
                      handover_count=protocol.handover_count, final_phase=protocol.phase,
                      elapsed_seconds=time.monotonic() - begun, index=index, warmup=index == 0,
                      open_calls=pairing.stacks, completed_intervals=pairing.completed)
        save('observer-result.json', output)
        if target is not None and (process is None or not process.IsValid() or process.GetState() == lldb.eStateExited):
            output['target_deleted'] = debugger.DeleteTarget(target)
            save('observer-result.json', output)
        print('B24_BOUNDARIES_' + output['state'], flush=True)
    return output


def run(debugger):
    global OUT
    session = E / 'boundaries'
    begun = time.monotonic()
    result = {'state': 'INCONCLUSIVE', 'targets': [], 'debugger_pid': os.getpid()}
    try:
        debugger.SetAsync(True)
        for command in ('settings set target.disable-aslr false',
                        'settings set target.skip-prologue false',
                        'settings set target.process.stop-on-sharedlibrary-events true'):
            response = lldb.SBCommandReturnObject()
            debugger.GetCommandInterpreter().HandleCommand(command, response)
            require(response.Succeeded(), response.GetError())
        for index in range(6):
            directory = session / ('target-' + str(index))
            while not (directory / 'launch-permit.json').exists():
                require(not (session / 'stop.json').exists(), 'outer terminal stop')
                require(time.monotonic() - begun < 510, 'embedded session elapsed guard')
                time.sleep(.01)
            permit = json.loads((directory / 'launch-permit.json').read_text())
            require(permit['index'] == index and permit['run_id'] == run_id(index)
                    and permit['full_target_seconds'] == 60, 'wrong launch permit')
            row = observe(debugger, index)
            result['targets'].append({'index': index, 'run_id': run_id(index),
                                      'state': row['state'], 'pid': row.get('pid')})
            require(row['state'] == 'ENTRIES_COMPLETE' and row.get('target_deleted'), 'target observer failed')
            while not (directory / 'next-ready.json').exists():
                require(not (session / 'stop.json').exists(), 'outer terminal stop')
                require(time.monotonic() - begun < 510, 'embedded session elapsed guard')
                time.sleep(.01)
            ready = json.loads((directory / 'next-ready.json').read_text())
            require(ready['index'] == index and ready['run_id'] == run_id(index)
                    and ready['state'] == 'PASS', 'wrong release permit')
        result['state'] = 'COMPLETE'
    except BaseException:
        result['error'] = traceback.format_exc()
    finally:
        from boundaries_core import save as save_path
        save_path(session / 'session-result.json', result)
