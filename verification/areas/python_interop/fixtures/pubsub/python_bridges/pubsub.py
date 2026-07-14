import asyncio
from types import SimpleNamespace


async def subscribe(handler):
    state = {"handler": handler, "closed": False, "active": set()}

    async def emit(value):
        if state["closed"]:
            raise RuntimeError("subscription is closed")
        task = asyncio.ensure_future(state["handler"](value))
        state["active"].add(task)
        try:
            return await task
        finally:
            state["active"].discard(task)

    async def aclose():
        state["closed"] = True
        if state["active"]:
            results = await asyncio.gather(*tuple(state["active"]))
            if results != [42]:
                raise RuntimeError(f"unexpected active callback results: {results!r}")
        state["handler"] = None

    eager = asyncio.ensure_future(emit(21))
    await asyncio.sleep(0)
    return SimpleNamespace(emit=emit, aclose=aclose, eager=eager)
