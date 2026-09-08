"""Explicit SDK-shaped R2 archive transport; no SDK factory or ambient access."""

from __future__ import annotations

import re
from typing import Any, Protocol

from .archive_r2_config import R2Target, _require
from .common import GovernanceError

MAX_SINGLE_PART_BYTES = 5 * 1024 ** 3
READ_CHUNK_BYTES = 1024 ** 2


class S3Client(Protocol):
    meta: Any

    def put_object(self, **kwargs: Any) -> dict: ...

    def get_object(self, **kwargs: Any) -> dict: ...


def validate_client(target: R2Target, client: S3Client) -> None:
    """Inspect standard SDK metadata; wire/TLS/credential proof belongs to F1C2."""
    try:
        meta = client.meta
        config = meta.config
        valid = (meta.endpoint_url == target.endpoint and config.region_name == "auto"
                 and config.s3 == {"addressing_style": "path"}
                 and isinstance(config.retries, dict)
                 and type(config.retries.get("total_max_attempts")) is int
                 and config.retries["total_max_attempts"] == 1
                 and set(config.retries) <= {"total_max_attempts", "mode"}
                 and config.request_checksum_calculation == "when_required"
                 and config.response_checksum_validation == "when_required")
    except Exception:
        raise GovernanceError("missing SDK target/configuration binding") from None
    _require(valid, "SDK endpoint/region/path/retry/checksum configuration mismatch")


def _status(response: Any) -> int | None:
    if not isinstance(response, dict):
        return None
    metadata = response.get("ResponseMetadata")
    if not isinstance(metadata, dict):
        return None
    status = metadata.get("HTTPStatusCode")
    return status if type(status) is int else None


def _error_parts(exc: Exception) -> tuple[int | None, str | None]:
    try:
        response = getattr(exc, "response", None)
        status = _status(response)
        error = response.get("Error") if isinstance(response, dict) else None
        code = error.get("Code") if isinstance(error, dict) else None
        return status, code if isinstance(code, str) else None
    except Exception:
        return None, None


def _failure(operation: str, exc: Exception) -> GovernanceError:
    status, code = _error_parts(exc)
    # Do not expose exception text, arbitrary message bodies, URLs or credentials.
    safe_status = str(status) if status is not None and 100 <= status <= 599 else "unknown"
    safe_code = code if code and re.fullmatch(r"[A-Za-z][A-Za-z0-9]{0,63}", code) else "redacted"
    return GovernanceError(f"R2 {operation} failed: HTTP {safe_status}, code {safe_code}")


class R2ArchiveReader:
    """Dedicated read client; this object exposes no create operation."""

    def __init__(self, target: R2Target, client: S3Client):
        validate_client(target, client)
        self.target = target
        self._client = client

    @property
    def location(self) -> str:
        return self.target.location

    def read(self, key: str) -> bytes:
        self.target.validate_key(key)
        validate_client(self.target, self._client)
        body = None
        try:
            response = self._client.get_object(Bucket=self.target.bucket, Key=key)
            if isinstance(response, dict):
                body = response.get("Body")
            _require(_status(response) == 200 and "Error" not in response
                     and "ContentRange" not in response and "DeleteMarker" not in response,
                     "R2 read requires a complete 200 response")
            headers = response["ResponseMetadata"].get("HTTPHeaders", {})
            _require(isinstance(headers, dict) and all(isinstance(key, str) for key in headers),
                     "R2 read has malformed response headers")
            _require(not {"content-range", "content-encoding", "x-amz-expiration"}.intersection(
                key.lower() for key in headers), "R2 read has range, encoding or expiration headers")
            _require(callable(getattr(body, "read", None)) and callable(getattr(body, "close", None)),
                     "R2 read has no complete stream")
            length = response.get("ContentLength")
            _require(type(length) is int and 0 <= length <= MAX_SINGLE_PART_BYTES,
                     "R2 read length is missing or over limit")
            _require(response.get("StorageClass") == "STANDARD"
                     and "ContentEncoding" not in response and "Expiration" not in response,
                     "R2 read has missing Standard metadata, encoding or expiration")
            chunks = []
            total = 0
            while True:
                # Read through EOF, including one byte beyond a declared length.
                chunk = body.read(min(READ_CHUNK_BYTES, length - total + 1))
                _require(type(chunk) is bytes, "R2 body returned non-bytes")
                if not chunk:
                    break
                total += len(chunk)
                _require(total <= length and total <= MAX_SINGLE_PART_BYTES, "R2 body exceeds declared length")
                chunks.append(chunk)
            _require(total == length, "R2 body was truncated")
            return b"".join(chunks)
        except Exception as exc:
            raise _failure("read", exc) from None
        finally:
            if body is not None:
                try:
                    body.close()
                except Exception:
                    raise GovernanceError("R2 body cleanup failed") from None


class R2ArchiveStore(R2ArchiveReader):
    """One conditional PUT per call; no existence probe, overwrite or retry."""

    def create(self, key: str, content: bytes) -> bool:
        self.target.validate_key(key)
        _require(isinstance(content, bytes) and len(content) <= MAX_SINGLE_PART_BYTES,
                 "R2 single-part content is invalid or over limit")
        # Reject byte subclasses with a forged length before crossing the SDK boundary.
        _require(type(content) is bytes, "R2 content must be exact bytes")
        validate_client(self.target, self._client)
        try:
            response = self._client.put_object(
                Bucket=self.target.bucket, Key=key, Body=content, ContentLength=len(content),
                IfNoneMatch="*", StorageClass="STANDARD",
            )
        except Exception as exc:
            if _error_parts(exc) == (412, "PreconditionFailed"):
                return False
            raise _failure("create", exc) from None
        _require(_status(response) == 200 and "Error" not in response
                 and isinstance(response.get("ETag"), str) and bool(response["ETag"]),
                 "R2 create did not return complete success")
        return True


class R2ReadbackStore:
    """F1B-compatible composition: independent reads plus receipt-only writes.

Credential references are binding labels, not proof of actual role separation.
The online owner must authenticate them without exposing credentials here.
"""

    def __init__(self, reader: R2ArchiveReader, receipt_writer: R2ArchiveStore | None):
        _require(type(reader) is R2ArchiveReader, "dedicated read-only handle required")
        _require(type(receipt_writer) is R2ArchiveStore, "separate receipt writer required")
        _require(reader.target.binding == receipt_writer.target.binding,
                 "receipt writer target mismatch")
        _require(reader._client is not receipt_writer._client
                 and reader.target.credential_ref != receipt_writer.target.credential_ref,
                 "reader and receipt writer must have separate clients/references")
        self._reader = reader
        self._writer = receipt_writer

    @property
    def location(self) -> str:
        return self._reader.location

    def read(self, key: str) -> bytes:
        return self._reader.read(key)

    def create(self, key: str, content: bytes) -> bool:
        self._reader.target.validate_key(key, receipt_only=True)
        _require(self._reader.target.binding == self._writer.target.binding, "receipt writer target mismatch")
        return self._writer.create(key, content)
