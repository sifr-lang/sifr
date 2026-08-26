import redis


def run(endpoint: str, token: str) -> str:
    key = f"sifr:live:{token}"
    counter = f"{key}:counter"
    left_set = f"{key}:left"
    right_set = f"{key}:right"
    client = redis.Redis.from_url(endpoint, decode_responses=True)
    try:
        if client.ping() is not True:
            raise RuntimeError("Redis ping did not return true")
        if client.set(key, token) is not True or client.get(key) != token:
            raise RuntimeError("Redis value did not round-trip")
        if client.incr(counter) != 1:
            raise RuntimeError("Redis counter did not round-trip")
        client.sadd(left_set, "alpha", "shared")
        client.sadd(right_set, "shared", "omega")
        if client.sdiffcard(2, [left_set, right_set]) != 1:
            raise RuntimeError("Redis SDIFFCARD returned the wrong cardinality")
        if client.sunioncard(2, [left_set, right_set]) != 3:
            raise RuntimeError("Redis SUNIONCARD returned the wrong cardinality")
        client.delete(key, counter, left_set, right_set)
    finally:
        client.close()
    return (
        "sifr-python-interop:live:redis:roundtrip=ok:"
        "difference=1:union=3:resources=zero"
    )
