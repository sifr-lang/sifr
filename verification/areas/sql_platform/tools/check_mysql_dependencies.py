"""Check the current driver's type identity and minimal dependency ownership."""

from __future__ import annotations

import copy
import tomllib
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[4]
DRIVER_VERSION = "0.37.1"
DRIVER_CHECKSUM = "40d11da0e2d9fad4640c9f9198ee431c6d68444568f83ef1f10f3367270071e4"
FEATURES = {"aws-lc-rs", "minimal-rust", "rustls-tls", "tls12"}
OWNERS = ("sifr_sql_dependency_lock", "sifr_sql_mysql", "sifr_sql_mysql_runtime", "sifr_sql_mysql_tools")


def load_inputs() -> dict[str, Any]:
    paths = {"root": "Cargo.toml", "lock": "Cargo.lock"}
    paths.update({owner: f"crates/{owner}/Cargo.toml" for owner in OWNERS})
    return {name: tomllib.loads((ROOT / path).read_text()) for name, path in paths.items()}


def validate(inputs: dict[str, Any]) -> list[str]:
    errors = []
    dependencies = inputs["root"]["workspace"]["dependencies"]
    if dependencies.get("mysql_async") != {"version": f"={DRIVER_VERSION}", "default-features": False}:
        errors.append("MySQL driver must use the exact current version with default features disabled")
    if "mysql_common" in dependencies:
        errors.append("MySQL protocol types must be owned through the driver, not a root common pin")
    for owner in OWNERS:
        manifest = inputs[owner]
        for table in ("dependencies", "dev-dependencies", "build-dependencies"):
            for alias, spec in manifest.get(table, {}).items():
                if alias == "mysql_common" or isinstance(spec, dict) and spec.get("package") == "mysql_common":
                    errors.append(f"{owner} has a direct mysql_common dependency")
        dependency = manifest.get("dependencies", {}).get("mysql_async")
        if dependency is None:
            dependency = manifest.get("dev-dependencies", {}).get("mysql_async", {})
        if (
            dependency.get("workspace") is not True
            or dependency.get("default-features", False) is not False
            or set(dependency.get("features", [])) != FEATURES
        ):
            errors.append(f"{owner} must select exactly the minimal MySQL TLS features")
    lock_owner = inputs["sifr_sql_dependency_lock"]
    if set(lock_owner["features"]["mysql"]) != {"dep:lalrpop", "dep:lalrpop-util", "dep:mysql_async"}:
        errors.append("MySQL qualification feature must not retain a common/derive anchor")
    packages = inputs["lock"]["package"]
    drivers = [row for row in packages if row["name"] == "mysql_async"]
    commons = [row for row in packages if row["name"] == "mysql_common"]
    if len(drivers) != 1 or drivers[0].get("version") != DRIVER_VERSION or drivers[0].get("checksum") != DRIVER_CHECKSUM:
        errors.append("Cargo.lock must select exactly the current canonical MySQL driver")
    if len(commons) != 1:
        errors.append("Cargo.lock must preserve one MySQL protocol type identity")
    elif len(drivers) == 1:
        common = commons[0]
        # Published mysql_async 0.37.1 requires ^0.37.1, not the latest 0.38 family.
        parts = str(common["version"]).split(".")
        if len(parts) != 3 or parts[:2] != ["0", "37"] or not parts[2].isdigit() or int(parts[2]) < 1:
            errors.append("MySQL common selection violates the actual driver's ^0.37.1 requirement")
        if not any(edge in {"mysql_common", f"mysql_common {common['version']}"} for edge in drivers[0]["dependencies"]):
            errors.append("MySQL driver must own the locked common edge")
    for row in packages:
        if row["name"].startswith("sifr_") and any(edge.split()[0] == "mysql_common" for edge in row.get("dependencies", [])):
            errors.append(f"{row['name']} retains direct common ownership in Cargo.lock")
    if any(row["name"] == "mysql-common-derive" for row in packages):
        errors.append("MySQL lock retains the unused derive dependency")
    return errors


def self_test() -> int:
    inputs = load_inputs()
    if errors := validate(inputs):
        raise ValueError("\n".join(errors))
    mutations = []
    for field, value in (("version", "=0.37.0"), ("default-features", True)):
        candidate = copy.deepcopy(inputs)
        candidate["root"]["workspace"]["dependencies"]["mysql_async"][field] = value
        mutations.append(candidate)
    for owner in OWNERS:
        table = "dev-dependencies" if owner == "sifr_sql_mysql" else "dependencies"
        candidate = copy.deepcopy(inputs)
        candidate[owner][table]["mysql_async"]["features"].append("derive")
        mutations.append(candidate)
        candidate = copy.deepcopy(inputs)
        candidate[owner][table]["mysql_async"]["default-features"] = True
        mutations.append(candidate)
    candidate = copy.deepcopy(inputs)
    candidate["sifr_sql_mysql_runtime"]["dependencies"]["protocol"] = {"package": "mysql_common", "version": "0.38.2"}
    mutations.append(candidate)
    candidate = copy.deepcopy(inputs)
    common = next(row for row in candidate["lock"]["package"] if row["name"] == "mysql_common")
    candidate["lock"]["package"].append({**common, "version": "0.38.2"})
    mutations.append(candidate)
    candidate = copy.deepcopy(inputs)
    next(row for row in candidate["lock"]["package"] if row["name"] == "mysql_async")["dependencies"].remove("mysql_common")
    mutations.append(candidate)
    for candidate in mutations:
        if not validate(candidate):
            raise ValueError("MySQL current dependency checker accepted a required mutation")
    return len(mutations)
