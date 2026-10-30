"""Construction policy for standalone native SQL tools, separate from the compiler.

Syn is optimized in the normal development compiler because it executes in that
product. In the standalone SQL graph it only constructs procedural macros.
Keep target code unchanged and qualify this construction configuration explicitly.
"""
from __future__ import annotations

from copy import deepcopy
from pathlib import Path
import tempfile


HOST_BUILD_OPTIONS = (
    "--config", "profile.dev.build-override.opt-level=0",
    "--config", "profile.dev.package.syn.opt-level=0",
    "--config", "profile.dev.package.proc-macro2.opt-level=0",
)
TARGET_PROFILE = {
    "opt_level": "1", "debuginfo": 0, "debug_assertions": True,
    "overflow_checks": True, "test": False,
}
HOST_ONLY_SYNTAX = frozenset({"syn", "proc_macro2"})


class NativeProfileError(ValueError):
    """The selected native graph crossed its qualified construction boundary."""


def validate_native_profiles(messages: list[dict], target_dir: Path, target: str) -> dict:
    """Use explicit-target Cargo artifacts; no filename-based source exclusions."""
    owned = target_dir.resolve()
    counts = {"host": 0, "target": 0}
    seen = set()
    for artifact in messages:
        if artifact.get("reason") != "compiler-artifact":
            continue
        descriptor = artifact.get("target")
        if not isinstance(descriptor, dict):
            raise NativeProfileError("native artifact has no target descriptor")
        name = descriptor.get("name")
        package = artifact.get("package_id")
        kinds, features = descriptor.get("kind"), artifact.get("features")
        paths = artifact.get("filenames", [])
        if (not isinstance(name, str) or not name or not isinstance(package, str) or not package
                or not isinstance(kinds, list) or not kinds
                or not all(isinstance(kind, str) for kind in kinds)
                or not isinstance(features, list)
                or not all(isinstance(feature, str) for feature in features)
                or not isinstance(paths, list) or not paths):
            raise NativeProfileError("native artifact has no target identity or outputs")
        roles = set()
        for output in paths:
            try:
                relative = Path(output).resolve().relative_to(owned)
            except (TypeError, ValueError) as error:
                raise NativeProfileError("native artifact output is outside its owned target") from error
            if relative.parts[:2] == (target, "debug"):
                roles.add("target")
            elif relative.parts[:1] == ("debug",):
                roles.add("host")
            else:
                raise NativeProfileError("native artifact has an unexpected target/profile path")
        if len(roles) != 1:
            raise NativeProfileError("native artifact mixes host and target outputs")
        role = roles.pop()
        # Cargo can build one build script for distinct host/target feature sets.
        # They are different units even when package, target name and role match.
        identity = (package, name, role, tuple(kinds), tuple(sorted(features)))
        if identity in seen:
            raise NativeProfileError("native artifact identity is duplicated")
        seen.add(identity)
        profile = artifact.get("profile")
        if not isinstance(profile, dict):
            raise NativeProfileError("native artifact has no profile evidence")
        expected_profile = TARGET_PROFILE
        if role == "target":
            if name in HOST_ONLY_SYNTAX:
                raise NativeProfileError("SQL syntax construction dependency entered target code")
        else:
            expected_profile = TARGET_PROFILE | {
                "opt_level": "1" if name == "sha2" else "0", "debuginfo": "line-tables-only"}
        if (profile != expected_profile or any(type(profile.get(key)) is not type(value)
                                              for key, value in expected_profile.items())):
            raise NativeProfileError(f"native {role} profile differs from qualification: {name}")
        counts[role] += 1
    if not all(counts.values()):
        raise NativeProfileError("native qualification has incomplete host/target artifact evidence")
    return counts


def self_test() -> int:
    """Fail closed for graph, profile and evidence drift before expensive work."""
    with tempfile.TemporaryDirectory(prefix="sql-native-profile-") as root:
        directory = Path(root)
        target = "x86_64-unknown-linux-gnu"
        def artifact(name, role):
            return {"reason": "compiler-artifact", "package_id": f"pkg:{name}",
                    "target": {"name": name, "kind": ["lib"]}, "features": [],
                    "filenames": [str(directory / ("debug" if role == "host" else
                                                    f"{target}/debug") / f"lib{name}.rlib")],
                    "profile": TARGET_PROFILE | ({"opt_level": "0", "debuginfo": "line-tables-only"}
                                                 if role == "host" else {})}
        valid = [artifact("syn", "host"), artifact("sql_tool", "target")]
        assert validate_native_profiles(valid, directory, target) == {"host": 1, "target": 1}
        distinct_features = deepcopy(valid[0])
        distinct_features["features"] = ["full"]
        assert validate_native_profiles([*valid, distinct_features], directory, target) == {
            "host": 2, "target": 1}
        optimized_hash = artifact("sha2", "host")
        optimized_hash["profile"]["opt_level"] = "1"
        assert validate_native_profiles([*valid, optimized_hash], directory, target) == {
            "host": 2, "target": 1}
        invalid = [[], valid[:1], valid[1:], valid + [valid[1]],
                   [valid[0], artifact("syn", "target")],
                   [valid[0], artifact("proc_macro2", "target")]]
        for key, value in (("opt_level", "0"), ("debug_assertions", False),
                           ("overflow_checks", False), ("debuginfo", 2), ("test", True),
                           ("debug_assertions", 1)):
            rows = deepcopy(valid)
            rows[1]["profile"][key] = value
            invalid.append(rows)
        for paths in ([], [str(directory.parent / "outside.rlib")],
                      [str(directory / "release/libwrong.rlib")],
                      [*valid[0]["filenames"], *valid[1]["filenames"]]):
            rows = deepcopy(valid)
            rows[1]["filenames"] = paths
            invalid.append(rows)
        rows = deepcopy(valid)
        rows[0]["profile"]["opt_level"] = "2"
        invalid.append(rows)
        for key, value in (("debug_assertions", False), ("overflow_checks", False),
                           ("debuginfo", 0)):
            rows = deepcopy(valid)
            rows[0]["profile"][key] = value
            invalid.append(rows)
        rows = deepcopy(valid)
        del rows[1]["profile"]
        invalid.append(rows)
        for rows in invalid:
            try:
                validate_native_profiles(rows, directory, target)
            except NativeProfileError:
                continue
            raise NativeProfileError("native profile self-test accepted invalid evidence")
        return len(invalid)
