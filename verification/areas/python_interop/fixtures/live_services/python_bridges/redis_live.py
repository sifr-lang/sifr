import redis


def run(endpoint: str, token: str) -> str:
    key = f"sifr:live:{token}"
    counter = f"{key}:counter"
    client = redis.Redis.from_url(endpoint, decode_responses=True)
    try:
        if client.ping() is not True:
            raise RuntimeError("Redis ping did not return true")
        if client.set(key, token) is not True or client.get(key) != token:
            raise RuntimeError("Redis value did not round-trip")
        if client.incr(counter) != 1:
            raise RuntimeError("Redis counter did not round-trip")
        client.delete(key, counter)
    finally:
        client.close()
    return "sifr-python-interop:live:redis:roundtrip=ok:resources=zero"
