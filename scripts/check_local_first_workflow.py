#!/usr/bin/env python3
"""Guard local-first admission and preserve the intended event/profile coverage.

This deliberately checks the small event selector contract, not arbitrary GitHub
expressions. GitHub remains the authority for actual workflow admission.
"""

from __future__ import annotations

import copy
import json
import re
from pathlib import Path

from check_uv_toolchain import parse_workflows

WORKFLOW = ".github/workflows/local-first-validation.yml"
SELECTOR = "${{ fromJSON(needs.validation-selection.outputs.profiles) }}"
CANDIDATE = "${{ github.event.pull_request.merge_commit_sha || github.event.merge_group.head_sha || github.sha }}"


def validate(document: dict) -> list[str]:
    errors = []
    jobs = document["jobs"]
    for name, job in jobs.items():
        # Job-level if is evaluated before the strategy expands the matrix.
        if re.search(r"\bmatrix(?:\.|\[)", str(job.get("if", ""))):
            errors.append(f"{name}: job-level if cannot reference matrix")
    profile = jobs["local-first-profiles"]
    if "if" in profile:
        errors.append("profile coverage must be selected in the matrix, not a job condition")
    strategy = profile["strategy"]
    if strategy.get("fail-fast") is not True:
        errors.append("profile fail-fast must remain enabled")
    if strategy["matrix"]["profile"] != SELECTOR or profile.get("needs") != "validation-selection":
        errors.append("profiles must use the complete candidate-bound selector")
    if document.get("permissions") != {"contents": "read"}:
        errors.append("candidate validation must have read-only permissions")
    events = document.get("on", document.get("true", {}))
    if not {"pull_request", "merge_group", "push", "schedule", "workflow_dispatch"}.issubset(events):
        errors.append("required delivery and scheduled events must remain declared")
    if events.get("merge_group", {}).get("types") != ["checks_requested"]:
        errors.append("merge_group must validate its admitted queue candidate")
    for name, job in jobs.items():
        for step in job.get("steps", []):
            if step.get("uses", "").startswith("actions/checkout@"):
                options = step.get("with", {})
                if options.get("persist-credentials") is not False or options.get("ref") != CANDIDATE:
                    errors.append(f"{name}: checkout must bind the actual candidate without credentials")
    select = jobs.get("validation-selection", {})
    if not any(step.get("run") == "uv run --project verification --locked python scripts/select_ci_validation.py"
               and "if" not in step and not step.get("continue-on-error") for step in select.get("steps", [])):
        errors.append("selection must execute the canonical commit-bound selector")
    if not any(step.get("run") == 'bash scripts/run_all_tests.sh --profile "${{ matrix.profile }}"'
               and step.get('if') == "needs.validation-selection.outputs.reuse != 'true'" and not step.get('continue-on-error', False)
               for step in profile['steps']):
        errors.append("every fresh selected profile must execute the authoritative runner")
    for step in profile['steps']:
        expected_condition = ("needs.validation-selection.outputs.reuse == 'true'"
            if step.get('name') == 'Record exact-commit correctness reuse'
            else "needs.validation-selection.outputs.reuse != 'true'")
        if step.get('if') != expected_condition or step.get('continue-on-error'):
            errors.append('profile steps require complete exact-commit reuse or fresh blocking execution')
    if not any(step.get('name') == 'Record exact-commit correctness reuse' for step in profile['steps']):
        errors.append('profile reuse requires its explicit current marker')
    for name, preparation in (
        ("smoke-fuzz-property", "uv run --project verification --locked python -m sifr_verify.ci_smoke_setup"),
        ("compiler-component-targets", 'cargo fetch --locked'),
        ("sql-wasi-build", "cargo fetch --locked"),
    ):
        job = jobs[name]
        condition = "needs.validation-selection.outputs.reuse != 'true'"
        matrix_job = name == 'compiler-component-targets'
        if job.get('needs') != 'validation-selection' or (not matrix_job and job.get('if') != condition):
            errors.append(f'{name}: only independently verified exact-commit correctness reuse may suppress execution')
        if matrix_job:
            if 'if' in job:
                errors.append('component reuse must expand all four native matrix jobs')
            for step in job['steps']:
                if step.get('name') == 'Record exact-commit correctness reuse':
                    if step.get('if') != "needs.validation-selection.outputs.reuse == 'true'":
                        errors.append('component reuse requires its explicit current marker')
                elif not (step.get('if') == condition or str(step.get('if')).startswith(condition+' && matrix.target == ')):
                    errors.append('component heavy steps must execute unless exact-commit reuse is verified')
        steps = jobs[name]["steps"]
        commands = [step.get("run") for step in steps]
        if preparation not in commands or commands.index(preparation) >= len(steps) - 1:
            errors.append(f"{name}: locked preparation must precede assertions")
        elif steps[commands.index(preparation)].get('continue-on-error') or (
                'if' in steps[commands.index(preparation)] and not matrix_job):
            errors.append(f"{name}: preparation must be unconditional and blocking")
    hardening = jobs.get("fuzz-hardening", {})
    if select.get('permissions') != {'contents': 'read', 'actions': 'read'}:
        errors.append('reuse discovery requires read-only producer facts')
    if select.get('outputs', {}).get('reuse') != '${{ steps.select.outputs.reuse }}':
        errors.append('reuse selection must bind the canonical selector output')
    if not any(step.get('if') == "github.event_name == 'push'" and
               step.get('with', {}).get('path') == 'target/validation-candidate/main-reuse.json'
               for step in select.get('steps', [])):
        errors.append('main reuse must retain its independent decision artifact')
    if hardening.get("if") != "github.event_name == 'schedule'" or hardening.get("strategy", {}).get("matrix", {}).get("target") != [
            "parser", "lowering", "ownership", "diagnostics", "project_graph"]:
        errors.append("nightly must retain all five explicit fuzz targets")
    if not any(step.get("if") == "always()" and step.get("uses", "").startswith("actions/upload-artifact@")
               for step in hardening.get("steps", [])):
        errors.append("nightly must preserve failed/incomplete fuzz evidence")
    return errors


def validate_publisher(document: dict) -> list[str]:
    errors = []
    event = document.get("on", document.get("true", {}))
    if event != {"workflow_run": {"workflows": ["local-first-validation"], "types": ["completed"]}}:
        errors.append("publisher must use the trusted workflow_run definition")
    jobs = document.get("jobs", {})
    if set(jobs) != {"publish"} or document.get("permissions") != {"contents": "read"}:
        errors.append("publisher must isolate write permissions to its trusted job")
    job = jobs.get("publish", {})
    if job.get("environment") != "validation-check-publication":
        errors.append("App credentials require the protected publication environment")
    if job.get("permissions") != {"contents": "read", "actions": "read"}:
        errors.append("publisher Actions token must remain read-only")
    if not any(step.get("id") == "app-token" and step.get("uses") ==
               "actions/create-github-app-token@bcd2ba49218906704ab6c1aa796996da409d3eb1" and
               step.get("with", {}).get("app-id") == "${{ vars.VALIDATION_CHECK_APP_ID }}" and
               step.get("with", {}).get("private-key") == "${{ secrets.VALIDATION_CHECK_APP_PRIVATE_KEY }}" and
               step.get("with", {}).get("permission-checks") == "write" for step in job.get("steps", [])):
        errors.append("publisher requires a separate pinned protected-check integration")
    for step in job.get("steps", []):
        if step.get("uses", "").startswith("actions/checkout@"):
            if step.get("with", {}).get("ref") != "${{ github.sha }}" or step.get("with", {}).get("persist-credentials") is not False:
                errors.append("publisher must never checkout candidate source")
    if not any(step.get("run") == "uv run --project verification --locked python scripts/publish_validation_aggregate.py"
               and "if" not in step and not step.get("continue-on-error") for step in job.get("steps", [])):
        errors.append("publisher must independently reconcile producer facts")
    return errors


def validate_native_qualification(parent: dict, document: dict) -> list[str]:
    errors = []
    job = parent['jobs'].get('published-predecessor-qualification', {})
    if job != {'if': "github.event_name == 'workflow_dispatch'",
               'uses': './.github/workflows/published-native-qualification.yml'}:
        errors.append('native qualification must use the explicit manual reusable workflow')
    if document.get('permissions') != {'contents': 'read'} or document.get('on', document.get('true')) != {'workflow_call': None, 'workflow_dispatch': None}:
        errors.append('native qualification must remain reusable and read-only')
    native = document['jobs']['native']
    expected = [('aarch64-apple-darwin', 'macos-15-xlarge'), ('x86_64-apple-darwin', 'macos-15-large'),
                ('x86_64-unknown-linux-gnu', 'ubuntu-24.04'), ('aarch64-unknown-linux-gnu', 'ubuntu-24.04-arm')]
    matrix = native.get('strategy', {}).get('matrix', {}).get('include', [])
    if [(row.get('target'), row.get('runner')) for row in matrix] != expected or native.get('runs-on') != '${{ matrix.runner }}':
        errors.append('native qualification requires all four actual host targets')
    if ([row.get('host_kind', '') for row in matrix] != ['dedicated-darwin', 'dedicated-darwin', '', '']
            or native.get('env') != {'SIFR_NATIVE_HOST_KIND': "${{ matrix.host_kind || '' }}"}):
        errors.append('native admission must declare exactly the Darwin hosts using job-allowed matrix context')
    steps = native.get('steps', [])
    checkout = next((step for step in steps if step.get('uses', '').startswith('actions/checkout@')), {})
    if checkout.get('with') != {'ref': '${{ github.sha }}', 'fetch-depth': 0, 'submodules': 'recursive', 'persist-credentials': False}:
        errors.append('native qualification must bind the exact committed source without credentials')
    for phrase in ('native_source_dependencies.py prepare', 'native_source_dependencies.py check', 'native_candidate.py prepare', 'prepare(policy=policy', 'qualify(root/', 'check(root/'):
        if not any(phrase in step.get('run', '') and 'if' not in step and not step.get('continue-on-error') for step in steps):
            errors.append('native qualification requires blocking preparation and transition checking: '+phrase)
    if not any(step.get('if') == 'always()' and step.get('uses', '').startswith('actions/upload-artifact@') for step in steps):
        errors.append('native qualification must preserve failed evidence and exact bundles')
    return errors


def validate_capacity_diagnostics(document: dict) -> list[str]:
    errors = []
    if (document.get('on', document.get('true')) != {'workflow_dispatch': None}
            or document.get('permissions') != {'contents': 'read'}
            or set(document.get('jobs', {})) != {'observe'}):
        errors.append('capacity diagnostics must be manual and read-only')
    job = document.get('jobs', {}).get('observe', {})
    rows = job.get('strategy', {}).get('matrix', {}).get('include', [])
    if ([(r.get('target'), r.get('runner')) for r in rows] != [
            ('aarch64-apple-darwin', 'macos-15'), ('x86_64-apple-darwin', 'macos-15-intel')]
            or job.get('runs-on') != '${{ matrix.runner }}'
            or job.get('strategy', {}).get('fail-fast') is not False
            or job.get('timeout-minutes') != 15
            or job.get('env') != {'SIFR_NATIVE_HOST_KIND': 'dedicated-darwin'}
            or 'permissions' in job or 'environment' in job):
        errors.append('capacity diagnostics require both bounded standard Darwin hosts')
    steps = job.get('steps', [])
    commands = ['uv python install 3.14.7', *[
        'uv run --project verification --locked python verification/areas/sysroot_release/native_capacity_diagnostics.py '
        + mode + ' --target "${{ matrix.target }}" --output "$RUNNER_TEMP/native-capacity-diagnostics"'
        for mode in ('observe', 'check')]]
    if (len(steps) != 6 or [s.get('run') for s in steps if 'run' in s] != commands
            or any('if' in s or s.get('continue-on-error') for s in steps[:-1])):
        errors.append('capacity diagnostics must execute only observation and independent checking')
    checkout = steps[0] if steps else {}
    if (not checkout.get('uses', '').startswith('actions/checkout@')
            or checkout.get('with') != {'ref': '${{ github.sha }}', 'persist-credentials': False}):
        errors.append('capacity diagnostics must bind exact source without credentials')
    upload = steps[-1] if steps else {}
    if (upload.get('if') != 'always()' or upload.get('continue-on-error')
            or not upload.get('uses', '').startswith('actions/upload-artifact@')
            or upload.get('with') != {
                'name': 'native-capacity-${{ github.sha }}-${{ matrix.target }}-${{ github.run_attempt }}',
                'path': '${{ runner.temp }}/native-capacity-diagnostics', 'if-no-files-found': 'error'}):
        errors.append('capacity diagnostics must retain source-bound raw facts and failures')
    return errors


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    document = parse_workflows({WORKFLOW: (root / WORKFLOW).read_text()})[WORKFLOW]
    publisher_path = ".github/workflows/validation-required.yml"
    publisher = parse_workflows({publisher_path: (root / publisher_path).read_text()})[publisher_path]
    native_path = '.github/workflows/published-native-qualification.yml'
    native = parse_workflows({native_path: (root / native_path).read_text()})[native_path]
    diagnostic_path = '.github/workflows/native-capacity-diagnostics.yml'
    diagnostic = parse_workflows({diagnostic_path: (root / diagnostic_path).read_text()})[diagnostic_path]
    errors = (validate(document) + validate_publisher(publisher)
              + validate_native_qualification(document, native) + validate_capacity_diagnostics(diagnostic))
    if errors:
        raise SystemExit("\n".join(errors))
    # Regression: the original condition must fail before any runner work.
    invalid = copy.deepcopy(document)
    invalid["jobs"]["local-first-profiles"]["if"] = (
        "github.event_name != 'pull_request' || matrix.profile == 'create-pr'"
    )
    assert any("cannot reference matrix" in error for error in validate(invalid))
    # A static or bypassed selector cannot make coverage appear green.
    narrowed = copy.deepcopy(document)
    narrowed["jobs"]["local-first-profiles"]["strategy"]["matrix"]["profile"] = ["create-pr"]
    assert any("candidate-bound selector" in error for error in validate(narrowed))
    wrong_checkout = copy.deepcopy(document)
    wrong_checkout["jobs"]["local-first-profiles"]["steps"][1]["with"]["ref"] = "main"
    assert any("actual candidate" in error for error in validate(wrong_checkout))
    skipped = copy.deepcopy(document)
    for step in skipped["jobs"]["local-first-profiles"]["steps"]:
        if step.get("name") == "Run local-first profile":
            step["if"] = "false"
    assert any("must execute" in error for error in validate(skipped))
    unprepared = copy.deepcopy(document)
    unprepared["jobs"]["sql-wasi-build"]["steps"] = [
        step for step in unprepared["jobs"]["sql-wasi-build"]["steps"]
        if step.get("run") != "cargo fetch --locked"
    ]
    assert any("preparation must precede" in error for error in validate(unprepared))
    bypassed_reuse = copy.deepcopy(document)
    bypassed_reuse['jobs']['smoke-fuzz-property']['if'] = "github.event_name != 'push'"
    assert any('only independently verified' in error for error in validate(bypassed_reuse))
    untrusted = copy.deepcopy(publisher)
    untrusted["jobs"]["publish"]["steps"][0]["with"]["ref"] = CANDIDATE
    assert any("never checkout candidate" in error for error in validate_publisher(untrusted))
    exposed = copy.deepcopy(publisher)
    exposed["jobs"]["publish"].pop("environment")
    assert any("protected publication environment" in error for error in validate_publisher(exposed))
    for mutation in ('target', 'runner', 'intel-runner', 'source', 'skip', 'write', 'host', 'context'):
        invalid_native = copy.deepcopy(native)
        job = invalid_native['jobs']['native']
        if mutation == 'target': job['strategy']['matrix']['include'].pop()
        if mutation == 'runner': job['strategy']['matrix']['include'][0]['runner'] = 'macos-15'
        if mutation == 'intel-runner': job['strategy']['matrix']['include'][1]['runner'] = 'macos-15-intel'
        if mutation == 'source': job['steps'][0]['with']['ref'] = 'main'
        if mutation == 'skip':
            for step in job['steps']:
                if 'qualify(root/' in step.get('run', ''): step['continue-on-error'] = True
        if mutation == 'write': invalid_native['permissions']['contents'] = 'write'
        if mutation == 'host': job['strategy']['matrix']['include'][0].pop('host_kind')
        if mutation == 'context': job['env']['SIFR_NATIVE_HOST_KIND'] = "${{ runner.os == 'macOS' && 'dedicated-darwin' || '' }}"
        assert validate_native_qualification(document, invalid_native), mutation
    for mutation in ('paid', 'source', 'skip', 'write', 'build', 'retention'):
        bad = copy.deepcopy(diagnostic); job = bad['jobs']['observe']
        if mutation == 'paid': job['strategy']['matrix']['include'][0]['runner'] = 'macos-15-xlarge'
        if mutation == 'source': job['steps'][0]['with']['ref'] = 'main'
        if mutation == 'skip': job['steps'][3]['if'] = 'false'
        if mutation == 'write': bad['permissions']['contents'] = 'write'
        if mutation == 'build': job['steps'].insert(-1, {'run': 'cargo build'})
        if mutation == 'retention': job['steps'][-1].pop('if')
        assert validate_capacity_diagnostics(bad), mutation
    print("local-first admission and event/profile contracts passed (including regressions)")


if __name__ == "__main__":
    main()
