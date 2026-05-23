def run_fixedbugs_suite(
    *,
    suite: dict[str, Any],
    repo_root: Path,
    actual_root: Path,
) -> dict[str, Any]:
    suite_name = suite["name"]
    index_raw = suite.get("index")
    if not isinstance(index_raw, str):
        raise SystemExit(f"suite '{suite_name}' missing string 'index'")
    index_path = repo_root / index_raw
    entries = load_index(index_path)
    if not entries:
        raise SystemExit(f"suite '{suite_name}' has empty index: {index_path}")
    print(f"  suite={suite_name} owner={suite.get('owner', 'unknown')} entries={len(entries)}")

    result = {
        "name": suite_name,
        "owner": suite.get("owner", "unknown"),
        "blocking": bool(suite.get("blocking", False)),
        "runner": "fixedbugs",
        "index": str(index_path.relative_to(repo_root)),
        "cases": [],
        "failed_cases": 0,
        "total_variants": 0,
        "total_failures": 0,
    }

    required = (
        "id",
        "issue",
        "root_cause_category",
        "suite_location",
        "command",
        "note",
    )

    for entry in entries:
        case_id = str(entry.get("id", "<missing-id>"))
        case_result = {
            "id": case_id,
            "issue": entry.get("issue"),
            "root_cause_category": entry.get("root_cause_category"),
            "entry": entry.get("suite_location"),
            "command": entry.get("command"),
            "variants": [],
        }
        case_failed = False

        missing = required_missing(entry, required)
        expected_exit = entry.get("expect_exit_code")
        command_name = entry.get("command")
        entry_path_raw = entry.get("suite_location")
        formats = parse_formats(entry.get("diagnostic_formats"))
        if not formats:
            formats = [None]

        metadata_mismatches = list(missing)
        if not isinstance(expected_exit, int):
            metadata_mismatches.append("expect_exit_code")
        if command_name not in BASELINE_COMMANDS:
            metadata_mismatches.append("command")
        entry_path = repo_root / str(entry_path_raw) if isinstance(entry_path_raw, str) else None
        if entry_path is None or not entry_path.is_file():
            metadata_mismatches.append("suite_location")

        if metadata_mismatches:
            case_failed = True
            result["total_failures"] += 1
            result["total_variants"] += 1
            case_result["variants"].append(
                {
                    "label": "metadata",
                    "status": "fail",
                    "mismatches": sorted(set(metadata_mismatches)),
                }
            )
            result["cases"].append(case_result)
            result["failed_cases"] += 1
            continue

        assert entry_path is not None
        for diagnostic_format in formats:
            label = baseline_variant_label(str(command_name), diagnostic_format)
            exit_code, stdout, stderr, elapsed_ms, argv = run_variant(
                repo_root=repo_root,
                command_name=str(command_name),
                entry=entry_path,
                diagnostic_format=diagnostic_format,
            )
            stdout_norm = canonicalize_output(
                repo_root=repo_root,
                text=stdout,
                diagnostic_format=diagnostic_format,
                stream="stdout",
            )
            stderr_norm = canonicalize_output(
                repo_root=repo_root,
                text=stderr,
                diagnostic_format=diagnostic_format,
                stream="stderr",
            )
            mismatches: list[str] = []
            if exit_code != expected_exit:
                mismatches.append("unexpected-exit")

            status = "pass" if not mismatches else "fail"
            result["total_variants"] += 1
            if mismatches:
                case_failed = True
                result["total_failures"] += 1
                actual_case_dir = actual_root / suite_name / case_id
                write_text(actual_case_dir / f"{label}.stdout.txt", stdout_norm)
                write_text(actual_case_dir / f"{label}.stderr.txt", stderr_norm)
                write_text(actual_case_dir / f"{label}.exit-code.txt", f"{exit_code}\n")

            case_result["variants"].append(
                {
                    "label": label,
                    "diagnostic_format": diagnostic_format,
                    "argv": argv,
                    "status": status,
                    "mismatches": mismatches,
                    "expected_exit_code": expected_exit,
                    "actual_exit_code": exit_code,
                    "duration_ms": round(elapsed_ms, 3),
                }
            )

        result["cases"].append(case_result)
        if case_failed:
            result["failed_cases"] += 1

    return result


def collect_fixedbug_ids(repo_root: Path, suites: list[dict[str, Any]]) -> set[str]:
    fixedbug_ids: set[str] = set()
    for suite in suites:
        if suite.get("runner", "baseline") != "fixedbugs":
            continue
        index_raw = suite.get("index")
        if not isinstance(index_raw, str):
            continue
        entries = load_index(repo_root / index_raw)
        for entry in entries:
            bug_id = entry.get("id")
            if isinstance(bug_id, str) and bug_id:
                fixedbug_ids.add(bug_id)
    return fixedbug_ids


def run_crashes_suite(
    *,
    suite: dict[str, Any],
    repo_root: Path,
    fixedbug_ids: set[str],
) -> dict[str, Any]:
    suite_name = suite["name"]
    index_raw = suite.get("index")
    if not isinstance(index_raw, str):
        raise SystemExit(f"suite '{suite_name}' missing string 'index'")
    index_path = repo_root / index_raw
    entries = load_index(index_path)
    if not entries:
        raise SystemExit(f"suite '{suite_name}' has empty index: {index_path}")
    print(f"  suite={suite_name} owner={suite.get('owner', 'unknown')} entries={len(entries)}")

    result = {
        "name": suite_name,
        "owner": suite.get("owner", "unknown"),
        "blocking": bool(suite.get("blocking", False)),
        "runner": "crashes",
        "index": str(index_path.relative_to(repo_root)),
        "cases": [],
        "failed_cases": 0,
        "total_variants": 0,
        "total_failures": 0,
        "unresolved_count": 0,
    }

    required = (
        "id",
        "issue",
        "owner",
        "status",
        "root_cause_category",
        "source_reference",
        "reproducer_fixture",
        "promotion_target_suite",
        "note",
    )

    for entry in entries:
        case_id = str(entry.get("id", "<missing-id>"))
        mismatches = required_missing(entry, required)
        status_raw = entry.get("status")
        source_ref = entry.get("source_reference")
        reproducer_ref = entry.get("reproducer_fixture")
        promotion_target = entry.get("promotion_target_suite")

        if status_raw not in {"unresolved", "promoted"}:
            mismatches.append("status")
        if status_raw == "unresolved":
            result["unresolved_count"] += 1
        if not isinstance(source_ref, str) or not (repo_root / source_ref).is_file():
            mismatches.append("source_reference")
        if not isinstance(reproducer_ref, str) or not (repo_root / reproducer_ref).is_file():
            mismatches.append("reproducer_fixture")
        if promotion_target != "fixedbugs":
            mismatches.append("promotion_target_suite")
        if status_raw == "promoted":
            promoted_id = entry.get("promoted_fixedbug_id")
            if not isinstance(promoted_id, str) or promoted_id not in fixedbug_ids:
                mismatches.append("promoted_fixedbug_id")

        variant_status = "pass" if not mismatches else "fail"
        result["total_variants"] += 1
        if mismatches:
            result["total_failures"] += 1
            result["failed_cases"] += 1

        result["cases"].append(
            {
                "id": case_id,
                "issue": entry.get("issue"),
                "status": status_raw,
                "root_cause_category": entry.get("root_cause_category"),
                "source_reference": source_ref,
                "reproducer_fixture": reproducer_ref,
                "promotion_target_suite": promotion_target,
                "variants": [
                    {
                        "label": "metadata",
                        "status": variant_status,
                        "mismatches": sorted(set(mismatches)),
                    }
                ],
            }
        )

    if result["unresolved_count"] == 0:
        result["total_failures"] += 1
        result["failed_cases"] += 1
        result["total_variants"] += 1
        result["cases"].append(
            {
                "id": "sentinel-unresolved-count",
                "variants": [
                    {
                        "label": "policy",
                        "status": "fail",
                        "mismatches": ["missing-unresolved-sentinels"],
                    }
                ],
            }
        )

    return result


def contains_internal_panic(text: str) -> bool:
    lowered = text.lower()
    return "internal compiler panic" in lowered or "panicked at" in lowered


