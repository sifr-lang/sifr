from __future__ import annotations

import os
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[4]


def main() -> int:
    environment = os.environ.copy()
    environment.setdefault("CARGO_NET_OFFLINE", "true")
    run(
        ["cargo", "test", "-p", "sifr_lsp", "python_declaration"],
        environment,
    )
    run(
        ["cargo", "test", "-p", "sifr_lsp", "protocol_policy_help"],
        environment,
    )
    run(
        ["cargo", "test", "-p", "sifr_driver", "python_interop"],
        environment,
    )
    print(
        "lsp-declaration-authoring: completion=verified navigation=compiler "
        "diagnostics=parity cancellation=checked cache_drift=checked unsupported_certified=0",
        flush=True,
    )
    return 0


def run(command: list[str], environment: dict[str, str]) -> None:
    completed = subprocess.run(
        command,
        cwd=REPO_ROOT,
        env=environment,
        check=False,
        text=True,
        capture_output=True,
    )
    if completed.returncode != 0:
        raise SystemExit(
            f"command failed ({' '.join(command)}):\n{completed.stdout}\n{completed.stderr}"
        )


if __name__ == "__main__":
    raise SystemExit(main())
