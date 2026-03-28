#!/usr/bin/env python3

from __future__ import annotations

from pathlib import Path

from phase31_leetcode_lib import REPO_ROOT, write_json
from phase31_leetcode_taxonomy import write_markdown
from phase31_leetcode_remediation import build_backlog, build_backlog_markdown


def main() -> None:
    backlog = build_backlog()
    backlog_markdown = build_backlog_markdown(backlog)
    verification_dir = REPO_ROOT / "verification" / "leetcode"
    backlog_json_path = verification_dir / "phase31_remediation_backlog.json"
    backlog_md_path = verification_dir / "phase31_remediation_backlog.md"

    write_json(backlog_json_path, backlog)
    write_markdown(backlog_md_path, backlog_markdown)

    print(f"wrote {backlog_json_path.relative_to(REPO_ROOT)}")
    print(f"wrote {backlog_md_path.relative_to(REPO_ROOT)}")


if __name__ == "__main__":
    main()
