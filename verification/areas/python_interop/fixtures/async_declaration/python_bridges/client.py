import asyncio
import threading

import httpx
from fastapi import FastAPI


_app = FastAPI()
_close_count = 0


@_app.get("/status")
async def _status():
    await asyncio.sleep(0)
    return {"status": 207, "message": "async-ready"}


class Client(httpx.AsyncClient):
    def __init__(self):
        super().__init__(
            transport=httpx.ASGITransport(app=_app),
            base_url="http://sifr.invalid",
        )

    async def aclose(self):
        global _close_count
        await super().aclose()
        _close_count += 1


async def make_client():
    await asyncio.sleep(0)
    return Client()


async def get_status(client, path):
    response = await client.get(path)
    payload = response.json()
    return {
        "status": payload["status"],
        "message": payload["message"],
    }


async def loop_identity():
    await asyncio.sleep(0)
    return f"{id(asyncio.get_running_loop())}:{threading.get_ident()}"


async def close_count():
    await asyncio.sleep(0)
    return _close_count


async def fail():
    await asyncio.sleep(0)
    raise ValueError("async declaration fixture failure")


async def wrong_output():
    await asyncio.sleep(0)
    return "not-an-int"
