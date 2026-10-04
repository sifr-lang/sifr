"""Exact-commit reuse of prior complete merge validation at the identical commit."""
from datetime import datetime, timezone
import hashlib
import io
import json
import os
import urllib.request
import zipfile

from validation_aggregate_policy import WORKFLOW, PLATFORMS, evaluate
from validation_candidate_artifact import candidate_identity, archive_bytes

REUSED_JOBS = frozenset({'local-first-merge', 'smoke-fuzz-property', 'sql-build-wasm32-wasip2'} |
                       {'compiler-component-' + target for target in PLATFORMS})


def api(path):
    request = urllib.request.Request(os.environ.get('GITHUB_API_URL', 'https://api.github.com') + path,
        headers={'Authorization': 'Bearer ' + os.environ['GH_TOKEN'], 'Accept': 'application/vnd.github+json'})
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.load(response)


def jobs_for(api, prefix, run):
    jobs = []
    for page in range(1, 101):
        response = api(f"{prefix}/actions/runs/{run['id']}/attempts/{run['run_attempt']}/jobs?per_page=100&page={page}")
        jobs.extend(response['jobs'])
        if len(jobs) == response['total_count']: return jobs
        if len(jobs) > response['total_count'] or not response['jobs']: break
    raise ValueError('prior producer job inventory is incomplete')


def verify(api, repository, candidate, source_id, *, now, identity_reader=candidate_identity):
    prefix = '/repos/' + repository
    run = api(f'{prefix}/actions/runs/{source_id}')
    if run.get('event') not in {'merge_group', 'pull_request'}:
        raise ValueError('main reuse requires a pre-merge producer, never recursive push evidence')
    workflow = api(prefix + '/actions/workflows/local-first-validation.yml')
    identity = identity_reader(api, prefix, run)
    if identity['candidate_sha'] != candidate:
        raise ValueError('prior producer executed a different exact commit; no cross-commit equivalence inferred')
    if run['event'] == 'pull_request':
        snapshots = run.get('pull_requests', [])
        if len(snapshots) != 1: raise ValueError('prior PR provenance is ambiguous')
        merged = api(prefix + '/pulls/' + str(snapshots[0]['number']))
        if (not merged.get('merged_at') or merged.get('merge_commit_sha') != candidate
                or merged['head']['sha'] != run['head_sha']):
            raise ValueError('prior PR was not merged as the exact required commit')
    observed = api(prefix + '/contents/' + WORKFLOW + '?ref=' + candidate)
    protected = api(prefix + '/contents/' + WORKFLOW + '?ref=main')
    errors = evaluate(run, jobs_for(api, prefix, run), candidate=candidate, profile='merge',
                      repository=repository, workflow_id=workflow['id'],
                      workflow_matches=observed.get('sha') == protected.get('sha'), now=now)
    if errors: raise ValueError('; '.join(errors))
    return {'source_run_id': run['id'], 'source_run_attempt': run['run_attempt'],
            'observed_candidate_sha': identity['candidate_sha'], 'reused_jobs': sorted(REUSED_JOBS)}


def select(api, repository, candidate, current_id, *, now, identity_reader=candidate_identity):
    # Bounded discovery. Missing/inaccessible/unknown provenance always runs fresh.
    response = api(f'/repos/{repository}/actions/workflows/local-first-validation.yml/runs?head_sha={candidate}&status=completed&per_page=100')
    reasons = []
    for run in response['workflow_runs']:
        if run['id'] >= current_id or run.get('event') not in {'merge_group', 'pull_request'}: continue
        try:
            return {'state': 'reused', 'candidate_sha': candidate,
                    **verify(api, repository, candidate, run['id'], now=now, identity_reader=identity_reader)}
        except (ValueError, KeyError, TypeError) as error:
            reasons.append(str(error))
    return {'state': 'fresh', 'candidate_sha': candidate, 'reused_jobs': [],
            'reason': reasons or ['no completed exact-commit pre-merge producer']}


def read_decision(api, prefix, run):
    name = f"validation-main-reuse-{run['run_attempt']}"
    inventory = api(f"{prefix}/actions/runs/{run['id']}/artifacts?name={name}&per_page=100")
    if inventory['total_count'] != 1 or len(inventory['artifacts']) != 1:
        raise ValueError('main reuse decision artifact is missing or duplicated')
    item = inventory['artifacts'][0];producer = item.get('workflow_run', {})
    if (item['name'] != name or item.get('expired') is not False
            or producer.get('id') != run['id'] or producer.get('head_sha') != run['head_sha']):
        raise ValueError('main decision belongs to a different producer')
    raw = archive_bytes(item['archive_download_url'])
    if item.get('digest') != 'sha256:' + hashlib.sha256(raw).hexdigest():
        raise ValueError('main decision bytes differ from GitHub digest')
    with zipfile.ZipFile(io.BytesIO(raw)) as archive:
        if archive.namelist() != ['main-reuse.json'] or archive.getinfo('main-reuse.json').file_size > 65536:
            raise ValueError('main decision inventory exceeds its bound')
        decision = json.loads(archive.read('main-reuse.json'))
    if (decision['candidate_sha'] != run['head_sha'] or decision['run_id'] != run['id']
            or decision['run_attempt'] != run['run_attempt'] or run.get('head_branch') != 'main'):
        raise ValueError('main decision identity differs')
    if decision['state'] == 'fresh' and decision['reused_jobs'] == []: return set()
    if decision['state'] != 'reused' or decision['source_run_id'] >= run['id']:
        raise ValueError('unknown or recursive main evidence')
    expected = verify(api, run['repository']['full_name'], run['head_sha'], decision['source_run_id'],
                      now=datetime.now(timezone.utc))
    if any(decision.get(key) != value for key, value in expected.items()):
        raise ValueError('prior producer facts differ from the declared main reuse decision')
    return set(expected['reused_jobs'])
