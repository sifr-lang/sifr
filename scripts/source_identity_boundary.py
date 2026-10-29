"""Keep manifest parsing behind the stateless identity crate's build feature."""

from copy import deepcopy


def manifest_failures(manifest: dict, aliases: dict[str, set[str]]) -> list[str]:
    failures = []
    found = False
    containers = [("root", manifest), *manifest.get("target", {}).items()]
    for owner, container in containers:
        for section in ("dependencies", "build-dependencies"):
            for alias, spec in container.get(section, {}).items():
                if "toml" not in aliases.get(alias, set()):
                    continue
                found = True
                if (owner != "root" or section != "dependencies" or alias != "toml"
                        or not isinstance(spec, dict) or spec.get("optional") is not True):
                    failures.append("sifr_identity: TOML must be an optional root dependency owned by build")
    if not found:
        return failures
    features = manifest.get("features", {})
    if features.get("build") != ["dep:toml"]:
        failures.append("sifr_identity: build must explicitly own dep:toml")
    for name, entries in features.items():
        if name != "build" and any(entry in {"toml", "dep:toml", "build"}
                                   or entry.startswith("toml/") for entry in entries):
            failures.append(f"sifr_identity: feature {name} activates build-only TOML")
    return failures


def validate(root, guard) -> list[str]:
    manifest = guard.load_manifest(root, "sifr_identity")
    aliases = guard.dependency_entries(manifest, root, include_dev=False)
    return manifest_failures(manifest, aliases)


def self_test() -> list[str]:
    valid = {"dependencies": {"sha2": {"workspace": True},
                              "toml": {"workspace": True, "optional": True}},
             "features": {"build": ["dep:toml"]},
             "dev-dependencies": {"toml": {"workspace": True}}}
    aliases = {"sha2": {"sha2"}, "toml": {"toml"}, "parser": {"toml"}}
    failures = manifest_failures(valid, aliases)
    mutations = []
    required = deepcopy(valid)
    required["dependencies"]["toml"].pop("optional")
    mutations.append(required)
    for name, entries in (("default", ["build"]), ("default", ["dep:toml"]),
                          ("runtime", ["toml"]), ("runtime", ["toml/parse"]),
                          ("build", [])):
        changed = deepcopy(valid)
        changed["features"][name] = entries
        mutations.append(changed)
    indirect = deepcopy(valid)
    indirect["features"].update(default=["manifest"], manifest=["build"])
    mutations.append(indirect)
    for section in ("dependencies", "build-dependencies"):
        changed = deepcopy(valid)
        changed["target"] = {"cfg(windows)": {section: {"toml": {"workspace": True, "optional": True}}}}
        mutations.append(changed)
    build_script = deepcopy(valid)
    build_script["build-dependencies"] = {"toml": {"workspace": True, "optional": True}}
    mutations.append(build_script)
    renamed = deepcopy(valid)
    renamed["dependencies"]["parser"] = renamed["dependencies"].pop("toml")
    mutations.append(renamed)
    for index, mutation in enumerate(mutations):
        if not manifest_failures(mutation, aliases):
            failures.append(f"identity build boundary mutation {index} escaped detection")
    return failures
