import fakeredis
import hiredis
import redis


def run() -> str:
    client = fakeredis.FakeRedis(decode_responses=True)
    client.set("sifr:library:redis", "ready")
    observed = client.get("sifr:library:redis")
    reader = hiredis.Reader()
    reader.feed(b"+PONG\r\n")
    reply = reader.gets()
    if observed != "ready" or reply != b"PONG" or not redis.__version__:
        raise RuntimeError("Redis/fakeredis/hiredis full example failed")
    return "sifr-python-interop:redis-fakeredis:value=ready:reply=PONG"
