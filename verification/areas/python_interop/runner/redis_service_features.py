from __future__ import annotations

import warnings
from importlib.metadata import version

import fakeredis
import hiredis
from redis import Redis


def main() -> int:
    if version("redis") != "8.1.0":
        raise RuntimeError("Redis is not at the audited stable release")
    if version("fakeredis") != "2.37.1":
        raise RuntimeError("Fakeredis is not at the audited stable release")
    if version("hiredis") != "3.4.1":
        raise RuntimeError("Hiredis is not at the audited stable release")
    if version("testcontainers") != "4.15.0":
        raise RuntimeError("Testcontainers is not at the audited stable release")

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

    print(
        "python Redis service features ok: redis=8.1.0 fakeredis=2.37.1 "
        "hiredis=3.4.1 testcontainers=4.15.0"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
