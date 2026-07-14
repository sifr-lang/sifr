import asyncio
from types import SimpleNamespace

_last_close_result = 0


async def last_close_result():
    return _last_close_result


async def subscribe(handler):
    state = {
        "handler": handler,
        "closed": False,
        "active": set(),
        "started": asyncio.Event(),
        "release": asyncio.Event(),
    }

    async def emit(value):
        if state["closed"]:
            raise RuntimeError("subscription is closed")

        async def held_callback():
            state["started"].set()
            await state["release"].wait()
            return await state["handler"](value)

        task = asyncio.ensure_future(held_callback())
        state["active"].add(task)
        try:
            return await task
        finally:
            state["active"].discard(task)

    async def aclose():
        global _last_close_result
        state["closed"] = True
        if not state["active"]:
            raise RuntimeError("close did not observe an active callback")
        state["release"].set()
        results = await asyncio.gather(*tuple(state["active"]))
        if results != [42]:
            raise RuntimeError(f"unexpected active callback results: {results!r}")
        state["handler"] = None
        _last_close_result = results[0]

    eager = asyncio.ensure_future(emit(21))
    await state["started"].wait()
    return SimpleNamespace(emit=emit, aclose=aclose, eager=eager)
