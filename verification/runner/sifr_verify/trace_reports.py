"""Reconcile diagnostic spans without granting child output gate authority."""
from __future__ import annotations
import json

PREFIXES = {"[sifr-command] ": "commands", "[sifr-e2e-event] ": "e2e_spans",
            "[sifr-e2e-shell] ": "shell_commands"}


def nonnegative(value):
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0


class Traces:
    def __init__(self):
        self.rows = {}
        self.observations = []
        self.errors = []

    def add(self, line, *, child=False):
        for prefix, kind in PREFIXES.items():
            if not line.startswith(prefix):
                continue
            if kind == "commands" and child:
                kind = "observed_child_commands"
            try:
                row = json.loads(line[len(prefix):])
                if not isinstance(row, dict):
                    raise ValueError("event is not an object")
                if kind != "shell_commands" and row.get("schema") != 1:
                    raise ValueError("unsupported event schema")
                event = row.get("event")
                if event in ("blocked", "fixture_failure") and kind == "e2e_spans":
                    self.observations.append(row)
                    return True
                if event == "heartbeat":
                    return True  # Live progress; not another elapsed-time contribution.
                if event not in ("start", "end") or not isinstance(row.get("id"), str):
                    raise ValueError("missing event or span identity")
                if not isinstance(row.get("scope"), str):
                    raise ValueError("missing scope")
                pair = self.rows.setdefault((kind, row["id"]), {})
                if event in pair:
                    pair["invalid"] = True
                    raise ValueError("duplicate span event")
                if event == "end" and "start" not in pair:
                    pair["invalid"] = True
                pair[event] = row
            except (ValueError, TypeError) as error:
                self.errors.append({"kind":kind,"error":str(error)})
            return True
        return False

    def report(self):
        result = {kind: [] for kind in (*PREFIXES.values(), "observed_child_commands")}
        for (kind, identity), pair in self.rows.items():
            start, end = pair.get("start"), pair.get("end")
            row = {"id":identity,"start":start,"end":end,"status":"incomplete"}
            valid = bool(start and end and not pair.get("invalid") and start["scope"] == end["scope"])
            if valid and kind == "e2e_spans":
                valid = (nonnegative(start.get("clock_us")) and nonnegative(end.get("clock_us"))
                         and nonnegative(end.get("elapsed_us")) and start.get("parent") == end.get("parent")
                         and end["clock_us"] - start["clock_us"] == end["elapsed_us"])
                row["elapsed_ms"] = end.get("elapsed_us", 0) / 1000 if valid else None
            elif valid and kind == "shell_commands":
                valid = nonnegative(end.get("elapsed_seconds")) and isinstance(end.get("exit_code"), int) and not isinstance(end.get("exit_code"), bool)
                row["elapsed_ms"] = end.get("elapsed_seconds", 0) * 1000 if valid else None
                row["resolution_ms"] = 1000
            elif valid:
                valid = nonnegative(end.get("elapsed_ms"))
                if end.get("status") != "incomplete":
                    code = end.get("exit_code")
                    valid = valid and isinstance(code, int) and not isinstance(code, bool)
                    valid = valid and (end.get("status") == "pass") == (code == 0)
                row["elapsed_ms"] = end.get("elapsed_ms") if valid else None
            if valid:
                status = ("pass" if end["exit_code"] == 0 else "fail") if kind == "shell_commands" else end.get("status")
                if status in ("pass", "fail", "incomplete"):
                    row["status"] = status
                else:
                    valid = False
            row["valid_terminal_pair"] = valid
            result[kind].append(row)
        suites = []
        for row in result["e2e_spans"]:
            start = row["start"] or {}
            if start.get("scope") != "suite":
                continue
            children = [child for child in result["e2e_spans"]
                        if (child["start"] or {}).get("parent") == row["id"]]
            children.sort(key=lambda child: (child["start"] or {}).get("clock_us", 0))
            valid = row["valid_terminal_pair"] and row["status"] != "incomplete" and all(child["valid_terminal_pair"] and child["status"] != "incomplete" for child in children)
            last = start.get("clock_us", 0)
            total = 0
            if valid:
                for child in children:
                    begin, end = child["start"]["clock_us"], child["end"]["clock_us"]
                    if begin < last or end > row["end"]["clock_us"]:
                        valid = False
                        break
                    last = end
                    total += child["end"]["elapsed_us"]
            suites.append({"id":row["id"],"status":row["status"],
                "wall_ms":row.get("elapsed_ms"),
                "sequential_stage_ms":total / 1000 if valid else None,
                "unattributed_ms":(row["end"]["elapsed_us"] - total) / 1000 if valid else None,
                "accounting_valid":valid,"stage_count":len(children),
                "note":"Only direct non-overlapping stages contribute to this sum. Process, native-attempt and per-case times are included observations, not additive wall time."})
        result.update(e2e_accounting=suites, observations=self.observations, parse_errors=self.errors,
                      authority="Child spans are observations; functional/performance gate status comes from the profile runner.")
        return result
