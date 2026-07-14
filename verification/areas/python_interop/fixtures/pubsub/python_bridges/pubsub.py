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
            await asyncio.gather(*tuple(state["active"]), return_exceptions=True)
        state["handler"] = None

    return SimpleNamespace(emit=emit, aclose=aclose)
