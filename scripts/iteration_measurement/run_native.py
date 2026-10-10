"""Experimental measurement continuation; never publication qualification."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sys

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
            "harness_sha256": hashlib.sha256(override.read_bytes()).hexdigest(),
            "wrapper_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
            "change": "Strict cache inspect JSON/text assertions updated to current root and all eight owner/path/policy entries. Every other assertion and all selected native/source/corpus checks retained.",
        }
        report.write_text(json.dumps(payload, indent=2) + "\n")
