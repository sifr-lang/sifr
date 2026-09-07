from __future__ import annotations

import warnings

import fakeredis
import hiredis
from redis import Redis

from dependency_versions import runtime_version_marker


def main() -> int:
    versions = runtime_version_marker("redis", "fakeredis", "hiredis", "testcontainers")

    for command in ("sdiffcard", "sunioncard", "lmovem"):
        if not callable(getattr(Redis, command, None)):
            raise RuntimeError(f"Redis 8.1 command is unavailable: {command}")

    fake = fakeredis.FakeRedis(protocol=3, decode_responses=True)
    fake.zadd("ranked", {"first": 1.0})
    if fake.zpopmin("ranked") != ["first", 1.0] or fake.exists("ranked") != 0:
        raise RuntimeError("Fakeredis RESP3 sorted-set behavior drifted")

    reader = hiredis.Reader()
    reader.feed(b"%2\r\n+safe\r\n:1\r\n+count\r\n:2\r\n")
    if reader.gets() != {b"safe": 1, b"count": 2}:
        raise RuntimeError("Hiredis RESP3 map parsing drifted")

    # Only the Testcontainers import/API contract requires deprecations as errors.
    # Restore the caller's warning policy after this block; do not suppress warnings.
    with warnings.catch_warnings():
        warnings.simplefilter("error", DeprecationWarning)
        from testcontainers.community.redis import RedisContainer
        from testcontainers.core.container import DockerContainer
        from testcontainers.core.wait_strategies import LogMessageWaitStrategy

        if RedisContainer.__module__ != "testcontainers.community.redis":
            raise RuntimeError("Testcontainers Redis import is not canonical")
        if not callable(DockerContainer.with_envs) or not callable(
            DockerContainer.waiting_for
        ):
            raise RuntimeError("Testcontainers stable container API is unavailable")
        strategy = LogMessageWaitStrategy("Ready")
        if strategy.with_startup_timeout(60) is not strategy:
            raise RuntimeError("Testcontainers wait strategy configuration drifted")

    print(f"python Redis service features ok: {versions}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
