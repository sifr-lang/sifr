import fakeredis
import hiredis
import redis


def run() -> str:
    client = fakeredis.FakeRedis(protocol=3, decode_responses=True)
    client.set("sifr:library:redis", "ready")
    observed = client.get("sifr:library:redis")
    client.zadd("sifr:library:ranked", {"first": 1.0})
    popped = client.zpopmin("sifr:library:ranked")
    key_removed = client.exists("sifr:library:ranked") == 0
    reader = hiredis.Reader()
    reader.feed(b"%2\r\n+status\r\n+PONG\r\n+count\r\n:2\r\n")
    reply = reader.gets()
    if (
        observed != "ready"
        or popped != ["first", 1.0]
        or not key_removed
        or reply != {b"status": b"PONG", b"count": 2}
        or not redis.__version__
    ):
        raise RuntimeError("Redis/fakeredis/hiredis full example failed")
    return (
        "sifr-python-interop:redis-fakeredis:value=ready:"
        "zpop=first:key-removed=true:map-count=2"
    )
