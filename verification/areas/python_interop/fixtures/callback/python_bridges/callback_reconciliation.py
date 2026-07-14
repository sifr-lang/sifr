import asyncio
from types import SimpleNamespace


_provisional_cancelled = False
_enter_cancelled = False
_exit_called = False
_sync_manager = None
_sync_exit_called = False


def _record_cancellation(kind):
    def record(task):
        global _provisional_cancelled, _enter_cancelled
        if kind == "provisional":
            _provisional_cancelled = task.cancelled()
        else:
            _enter_cancelled = task.cancelled()

    return record


class Manager(SimpleNamespace):
    def __init__(self):
        self.handler = None

    async def install_fail(self, handler):
        task = asyncio.ensure_future(handler(1))
        task.add_done_callback(_record_cancellation("provisional"))
        await asyncio.sleep(0)
        raise RuntimeError("registration failed after callback start")

    async def install(self, handler):
        self.handler = handler

    async def __aenter__(self):
        task = asyncio.ensure_future(self.handler(2))
        task.add_done_callback(_record_cancellation("enter"))
        await asyncio.sleep(0)
        raise RuntimeError("context entry failed after callback start")

    async def __aexit__(self, _kind, _value, _traceback):
        global _exit_called
        _exit_called = True
        return False


async def make_manager():
    return Manager()


async def status():
    await asyncio.sleep(0)
    await asyncio.sleep(0)
    provisional = "provisional-cancelled" if _provisional_cancelled else "provisional-live"
    entered = "enter-cancelled" if _enter_cancelled else "enter-live"
    exited = "exit-called" if _exit_called else "exit-skipped"
    return f"{provisional}:{entered}:{exited}"


class SyncManager(SimpleNamespace):
    def __init__(self):
        self.handler = None

    def install(self, handler):
        self.handler = handler

    def __enter__(self):
        if self.handler(1) != 2:
            raise RuntimeError("sync callback returned the wrong value")
        raise RuntimeError("sync context entry failed after callback")

    def __exit__(self, _kind, _value, _traceback):
        global _sync_exit_called
        _sync_exit_called = True
        return False


def make_sync_manager():
    global _sync_manager
    _sync_manager = SyncManager()
    return _sync_manager


def sync_status():
    try:
        _sync_manager.handler(2)
    except Exception as error:
        callback = "sync-closed" if type(error).__name__ == "SifrCallbackClosedError" else type(error).__name__
    else:
        callback = "sync-live"
    exited = "sync-exit-called" if _sync_exit_called else "sync-exit-skipped"
    return f"{callback}:{exited}"
