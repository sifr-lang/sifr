"""Experimental measurement continuation; never publication qualification."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sys
import time

root = Path(sys.argv.pop(1)).resolve()
os.environ["SIFR_MEASUREMENT_REPO_ROOT"] = str(root)
os.chdir(root)
sys.path.insert(0, str(root / "scripts/distribution"))
override = Path(__file__).with_name("dxf_cache_cli_current_contract.py")
spec = importlib.util.spec_from_file_location("dxf_cache_cli_acceptance", override)
module = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = module
spec.loader.exec_module(module)
import qualify_native_package
# Keep every original assertion; cap each command and continue independent families.
original_run = qualify_native_package.NativePackage.run
def bounded_run(self, name, command, **kwargs):
    kwargs['deadline'] = min(kwargs.get('deadline', 900), 1790)
    return original_run(self, name, command, **kwargs)
qualify_native_package.NativePackage.run = bounded_run

def observed_qualify(self):
    failures = []
    def attempt(name, function):
        started = time.perf_counter()
        try:
            result = function()
        except Exception as error:
            failures.append({'category': name, 'error': str(error)})
            self.report.setdefault('family_attempts', []).append({'category': name, 'status': 'failed-incomplete', 'seconds': time.perf_counter()-started, 'error': str(error)})
            self.save()
            return None
        self.report.setdefault('family_attempts', []).append({'category': name, 'status': 'completed', 'seconds': time.perf_counter()-started})
        self.save()
        return result
    attempt('rustc-identity', lambda: self.run('rustc-identity', ['rustc','-vV'], deadline=90))
    attempt('cargo-identity', lambda: self.run('cargo-identity', ['cargo','-vV'], deadline=90))
    try:
        self.prepare_transport()
        binary = self.generation_checks()
    except Exception as error:
        self.report.update(status='fail', failure=str(error), remaining='Installation prerequisite failed; packaged-executable categories could not start.')
        self.save()
        return
    from native_dxf_contracts import qualify_dxf
    attempt('DXF cache, tracing and source contracts', lambda: qualify_dxf(self,binary))
    attempt('Native development and release profile assertions', lambda: self.native_profiles(binary))
    attempt('Source and installed metadata corpus', lambda: self.metadata_corpus(binary))
    self.report.update(status='fail' if failures else 'pass', family_failures=failures)
    self.save()
qualify_native_package.NativePackage.qualify = observed_qualify

output = Path(sys.argv[sys.argv.index("--output") + 1])
try:
    qualify_native_package.main()
finally:
    report = output / "native-package.json"
    if report.exists():
        payload = json.loads(report.read_text())
        status = payload.pop("status")
        payload["status"] = "experimental-" + status
        payload["experimental_measurement"] = {
            "qualification_eligible": False,
            "source_artifact_run": "38074052189",
            "canonical_protocol_status": status,
            "source_test_preparation": "Exact driver and LSP lib test graphs prepared separately in timed CI steps; all existing test selectors and execution deadlines retained.",
            "harness_sha256": hashlib.sha256(override.read_bytes()).hexdigest(),
            "wrapper_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
            "change": "Strict cache inspect JSON/text assertions updated to current root and all eight owner/path/policy entries. Every other assertion and all selected native/source/corpus checks retained.",
        }
        report.write_text(json.dumps(payload, indent=2) + "\n")
