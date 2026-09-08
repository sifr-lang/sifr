"""Create-only archive transport interface, sealing and append-only readback receipts.

There is deliberately no filesystem/cloud store implementation here. A provider
must implement atomic create-only writes and fresh exact-byte reads. Test doubles
prove logic only; provider retention, permissions and independent custody belong
to the subsequent online item.
"""

from __future__ import annotations

from typing import Any, Callable, Protocol

from .archive_manifest import parse_manifest, validate_manifest, verify_bytes, verify_payloads
from .common import GovernanceError, canonical_json_bytes, require_nonempty_string, sha256_bytes


class ArchiveStore(Protocol):
    @property
    def location(self) -> str:
        """Provider-qualified custody identity, not a claim of independence."""
        ...

    def create(self, key: str, content: bytes) -> bool:
        """Atomically create; return False if key exists, never replace any bytes."""
        ...

    def read(self, key: str) -> bytes:
        """Fresh read of the exact key; missing/unavailable reads must raise."""
        ...


def manifest_key(manifest: dict[str, Any]) -> str:
    return manifest["archive_root"] + "manifest.json"


def seal(store: ArchiveStore, manifest: dict[str, Any],
         read: Callable[[dict[str, Any]], bytes]) -> str:
    """Verify input, copy/reuse verified objects, then create the root once."""
    validate_manifest(manifest)
    verify_payloads(manifest, read)
    for record in manifest["artifacts"]:
        content = verify_bytes(record, read(record))
        # False only permits reuse of identical, fully read-back content objects.
        store.create(record["key"], content)
        verify_bytes(record, store.read(record["key"]))
    verify_payloads(manifest, lambda record: store.read(record["key"]))
    raw = canonical_json_bytes(manifest)
    if not store.create(manifest_key(manifest), raw):
        raise GovernanceError("root manifest already sealed; no overwrite or reseal")
    if store.read(manifest_key(manifest)) != raw:
        raise GovernanceError("manifest readback mismatch")
    return sha256_bytes(raw)


def readback(store: ArchiveStore, *, key: str, manifest_sha256: str,
             expected_source: str, expected_run: int, expected_attempt: int) -> dict[str, Any]:
    manifest = parse_manifest(store.read(key), manifest_sha256=manifest_sha256,
                              expected_source=expected_source, expected_run=expected_run,
                              expected_attempt=expected_attempt)
    if key != manifest_key(manifest):
        raise GovernanceError("manifest key mismatch")
    verify_payloads(manifest, lambda record: store.read(record["key"]))
    return manifest


def attest_readback(stores: list[ArchiveStore], *, key: str, receipt_id: str,
                    kind: str, manifest_sha256: str, expected_source: str,
                    expected_run: int, expected_attempt: int) -> dict[str, Any]:
    """Append a receipt only after fresh complete reads of every custody location.

    Names cannot establish physical independence. The bytes-only assurance is
    explicit; the online owner must independently authenticate those locations.
    Receipt IDs are unique caller-owned audit-event identifiers. A partially
    written receipt set raises; each extant receipt truthfully covers completed
    readbacks and never rewrites the sealed manifest.
    """
    require_nonempty_string(receipt_id, "receipt id")
    if kind not in {"copy", "readback"} or not stores or (kind == "copy" and len(stores) < 2):
        raise GovernanceError("copy requires at least two configured custody locations")
    locations = [require_nonempty_string(store.location, "store location") for store in stores]
    if len(set(locations)) != len(locations) or len({id(store) for store in stores}) != len(stores):
        raise GovernanceError("duplicate custody location")
    manifests = [readback(store, key=key, manifest_sha256=manifest_sha256,
                          expected_source=expected_source, expected_run=expected_run,
                          expected_attempt=expected_attempt) for store in stores]
    receipt = {
        "schema_version": 1, "kind": kind, "receipt_id": receipt_id,
        "manifest_sha256": manifest_sha256,
        "inventory_sha256": manifests[0]["inventory_sha256"],
        "source_commit": expected_source, "run_id": expected_run, "run_attempt": expected_attempt,
        "locations": sorted(locations), "assurance": "complete-byte-readback-only",
    }
    raw = canonical_json_bytes(receipt)
    receipt_key = manifests[0]["archive_root"] + "receipts/" + sha256_bytes(receipt_id.encode()) + ".json"
    for store in stores:
        if not store.create(receipt_key, raw):
            raise GovernanceError("append-only receipt identity already exists")
        if store.read(receipt_key) != raw:
            raise GovernanceError("receipt readback mismatch")
    return receipt
