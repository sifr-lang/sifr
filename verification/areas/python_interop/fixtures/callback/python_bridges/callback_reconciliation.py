import asyncio
from types import SimpleNamespace


_provisional_cancelled = False
_enter_cancelled = False
_exit_called = False
_enter_handler_error = False
_manager = None
_escaped_async_call_handler = None
_async_handler_error = False
_sync_manager = None
_sync_exit_called = False
_sync_handler_error = False
_escaped_sync_call_handler = None
_sync_call_handler_error = False


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
    global _manager
    _manager = Manager()
    return _manager


async def swallow_async_handler_error(handler):
    global _async_handler_error, _escaped_async_call_handler
    _escaped_async_call_handler = handler
    try:
        await handler(9)
    except Exception as error:
        _async_handler_error = type(error).__name__ == "SifrCallbackError"


async def status():
    await asyncio.sleep(0)
    await asyncio.sleep(0)
    provisional = "provisional-cancelled" if _provisional_cancelled else "provisional-live"
    entered = "enter-cancelled" if _enter_cancelled else "enter-live"
    try:
        await _manager.handler(3)
    except Exception as error:
        closed = "enter-closed" if type(error).__name__ == "SifrCallbackClosedError" else type(error).__name__
    else:
        closed = "enter-live"
    try:
        await _escaped_async_call_handler(4)
    except Exception as error:
        call = "call-closed" if type(error).__name__ == "SifrCallbackClosedError" else type(error).__name__
    else:
        call = "call-live"
    exited = "exit-called" if _exit_called else "exit-skipped"
    typed = "typed-observed" if _async_handler_error else "typed-missing"
    return f"{provisional}:{entered}:{closed}:{call}:{exited}:{typed}"


class SyncManager(SimpleNamespace):
    def __init__(self):
        self.handler = None

    def install(self, handler):
        self.handler = handler

    def __enter__(self):
        global _sync_handler_error
        _sync_handler_error = self.handler(1) == 2
        raise RuntimeError("sync context entry failed after callback")

    def __exit__(self, _kind, _value, _traceback):
        global _sync_exit_called
        _sync_exit_called = True
        return False


def make_sync_manager():
    global _sync_manager
    _sync_manager = SyncManager()
    return _sync_manager


def swallow_sync_handler_error(handler):
    global _escaped_sync_call_handler, _sync_call_handler_error
    _escaped_sync_call_handler = handler
    try:
        handler(8)
    except Exception as error:
        _sync_call_handler_error = type(error).__name__ == "SifrCallbackError"


def sync_status():
    try:
        _sync_manager.handler(2)
    except Exception as error:
        callback = "sync-closed" if type(error).__name__ == "SifrCallbackClosedError" else type(error).__name__
    else:
        callback = "sync-live"
    try:
        _escaped_sync_call_handler(3)
    except Exception as error:
        call = "sync-call-closed" if type(error).__name__ == "SifrCallbackClosedError" else type(error).__name__
    else:
        call = "sync-call-live"
    handler = "sync-handler-success" if _sync_handler_error else "sync-handler-missing"
    exited = "sync-exit-called" if _sync_exit_called else "sync-exit-skipped"
    typed = "typed-observed" if _sync_call_handler_error else "typed-missing"
    return f"{callback}:{handler}:{call}:{exited}:{typed}"
