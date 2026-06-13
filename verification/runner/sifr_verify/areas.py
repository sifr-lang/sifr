"""Verification area discovery."""

from __future__ import annotations

import argparse
import importlib.util
import json
from dataclasses import dataclass
from pathlib import Path

from .errors import DiscoveryError
from .paths import AREAS_DIR, REPO_ROOT
from .schemas import load_json, load_schema, validate_data


@dataclass(frozen=True)
class Area:
    name: str
    owner: str
    manifest_path: Path
    parallel_safe: bool
    resource_classes: tuple[str, ...]


def discover_areas(areas_dir: Path = AREAS_DIR) -> list[Area]:
    if not areas_dir.exists():
        return []
    if not areas_dir.is_dir():
        raise DiscoveryError(f"areas path is not a directory: {areas_dir}")

    schema = load_schema("area.schema.json")
    areas: list[Area] = []
    seen: set[str] = set()
    for manifest_path in sorted(areas_dir.glob("*/manifest.json")):
        payload = load_json(manifest_path)
        source = _display_path(manifest_path)
        validate_data(payload, schema, source=source)
        name = payload["name"]
        if name in seen:
            raise DiscoveryError(f"duplicate verification area name: {name}")
        if manifest_path.parent.name != name:
            raise DiscoveryError(
                f"area manifest name '{name}' must match directory '{manifest_path.parent.name}'"
            )
        seen.add(name)
        areas.append(
            Area(
                name=name,
                owner=payload["owner"],
                manifest_path=manifest_path,
                parallel_safe=payload["parallel_safe"],
                resource_classes=tuple(payload.get("resource_classes", [])),
            )
        )
    return areas


def area_by_name(name: str, areas_dir: Path = AREAS_DIR) -> Area:
    for area in discover_areas(areas_dir):
        if area.name == name:
            return area
    raise DiscoveryError(f"unknown verification area: {name}")


def check_areas(areas_dir: Path = AREAS_DIR) -> list[str]:
    return [str(area.manifest_path.relative_to(REPO_ROOT)) for area in discover_areas(areas_dir)]


def print_list() -> None:
    areas = [
        {
            "name": area.name,
            "owner": area.owner,
            "manifest": str(area.manifest_path.relative_to(REPO_ROOT)),
            "parallel_safe": area.parallel_safe,
            "resource_classes": list(area.resource_classes),
        }
        for area in discover_areas()
    ]
    print(json.dumps({"schema_version": 1, "areas": areas}, indent=2, sort_keys=True))


def run_area(args: argparse.Namespace) -> int:
    area = area_by_name(args.area)
    runner_path = area.manifest_path.with_name("runner.py")
    if not runner_path.is_file():
        raise DiscoveryError(f"verification area has no runner: {area.name}")

    spec = importlib.util.spec_from_file_location(f"sifr_verify_area_{area.name}", runner_path)
    if spec is None or spec.loader is None:
        raise DiscoveryError(f"could not load verification area runner: {runner_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    main = getattr(module, "main", None)
    if not callable(main):
        raise DiscoveryError(f"verification area runner has no callable main(): {runner_path}")

    runner_args: list[str] = []
    for suite in args.suite:
        runner_args.extend(["--suite", suite])
    if args.bless:
        runner_args.append("--bless")
    if args.hardening_summary:
        runner_args.append("--hardening-summary")
    if args.result_json:
        runner_args.extend(["--result-json", args.result_json])
    return int(main(runner_args))


def run_command(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(prog="sifr_verify areas")
    subcommands = parser.add_subparsers(dest="command", required=True)

    subcommands.add_parser("list", help="List discovered areas as JSON.")
    subcommands.add_parser("check", help="Validate all discovered area manifests.")

    run_parser = subcommands.add_parser("run", help="Run one verification area adapter.")
    run_parser.add_argument("--area", required=True, help="Area name to execute.")
    run_parser.add_argument("--suite", action="append", default=[], help="Area suite filter.")
    run_parser.add_argument("--bless", action="store_true", help="Update checked-in baselines.")
    run_parser.add_argument(
        "--hardening-summary",
        action="store_true",
        help="Emit a legacy hardening summary line for validation report parsing.",
    )
    run_parser.add_argument("--result-json", help="Area result JSON path.")

    args = parser.parse_args(argv)
    if args.command == "list":
        print_list()
        return 0
    if args.command == "check":
        for manifest in check_areas():
            print(f"area ok: {manifest}")
        return 0
    if args.command == "run":
        return run_area(args)
    parser.error(f"unsupported areas command: {args.command}")
    return 2


def _display_path(path: Path) -> str:
    try:
        return str(path.relative_to(REPO_ROOT))
    except ValueError:
        return str(path)
