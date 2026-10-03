"""Consume explicitly declared preparation bytes; invalid receipts fail closed."""
from pathlib import Path

from self_update_certification import CertificationError
from sifr_verify.prepared_sysroot import OWNER_VARIABLE, command
from sifr_verify.process_execution import execute


def prepared_output(kind: str, root: Path, env: dict[str, str]) -> Path | None:
    if OWNER_VARIABLE not in env:
        return None
    result = execute(command("consume", kind, root), cwd=root, env=env, deadline_seconds=120)
    if result.returncode or result.cause != "exit" or result.truncated:
        raise CertificationError(f"prepared {kind} output rejected: {result.stderr.decode(errors='replace')}")
    value = result.stdout.decode().strip()
    if not value or "\n" in value or not Path(value).is_absolute():
        raise CertificationError("prepared output did not report one absolute path")
    return Path(value)
