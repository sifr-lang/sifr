import asyncio
import contextlib
import sqlite3
import threading

import aiosqlite


_events = []
_loop_identities = set()
_enter_count = 0
_exit_count = 0
_close_count = 0
_python_suppressed = False
_sifr_truthy_seen = False
_exit_failure_seen = False
_secondary_exit_failure_seen = False


def _record(event):
    _events.append(event)


def _record_loop():
    loop = asyncio.get_running_loop()
    _loop_identities.add(f"{id(loop)}:{threading.get_ident()}")


class DatabaseSession(aiosqlite.Connection):
    def __init__(self, mode):
        super().__init__(lambda: sqlite3.connect(":memory:"), 64)
        self.mode = mode

    async def __aenter__(self):
        global _enter_count
        _record_loop()
        _record(f"aenter-start:{self.mode}")
        await self
        await self.execute("create table evidence (value text not null)")
        await self.execute("insert into evidence(value) values ('sqlite-ready')")
        await self.commit()
        _enter_count += 1
        _record(f"aenter:{self.mode}")
        return self

    async def fetch_value(self):
        _record_loop()
        cursor = await self.execute("select value from evidence")
        try:
            row = await cursor.fetchone()
            return row[0]
        finally:
            await cursor.close()

    async def __aexit__(self, exc_type, exc, traceback):
        global _exit_count
        global _close_count
        global _python_suppressed
        global _sifr_truthy_seen
        global _exit_failure_seen
        global _secondary_exit_failure_seen

        _record_loop()
        cause = "None" if exc_type is None else exc_type.__name__
        _exit_count += 1
        _record(f"aexit:{self.mode}:{cause}")
        await self.close()
        _close_count += 1
        _record(f"closed:{self.mode}")

        if self.mode == "suppress-python":
            _python_suppressed = exc_type is ValueError
            return True
        if self.mode == "truthy-sifr":
            _sifr_truthy_seen = cause == "SifrBoundaryError"
            return True
        if self.mode == "exit-failure":
            _exit_failure_seen = True
            raise RuntimeError("async context exit failure")
        if self.mode == "secondary-exit-failure":
            _secondary_exit_failure_seen = exc_type is ValueError
            raise RuntimeError("secondary async context exit failure")
        return False


class SyncMarker(contextlib.AbstractContextManager):
    def __enter__(self):
        _record("sync-enter:nested")
        return self

    def __exit__(self, exc_type, exc, traceback):
        _record("sync-exit:nested")
        return False


async def reset():
    global _events
    global _loop_identities
    global _enter_count
    global _exit_count
    global _close_count
    global _python_suppressed
    global _sifr_truthy_seen
    global _exit_failure_seen
    global _secondary_exit_failure_seen

    _events = []
    _loop_identities = set()
    _enter_count = 0
    _exit_count = 0
    _close_count = 0
    _python_suppressed = False
    _sifr_truthy_seen = False
    _exit_failure_seen = False
    _secondary_exit_failure_seen = False
    _record_loop()


async def make_session(mode):
    _record_loop()
    await asyncio.sleep(0)
    return DatabaseSession(mode)


async def make_sync_marker():
    _record_loop()
    await asyncio.sleep(0)
    return SyncMarker()


async def originating_python_failure():
    _record_loop()
    await asyncio.sleep(0)
    raise ValueError("originating Python body failure")


async def mark(label):
    _record_loop()
    _record(label)
    await asyncio.sleep(0)


async def hold_until_cancelled():
    _record_loop()
    _record("hold-start:cancel")
    current = asyncio.current_task()
    if current is None:
        raise RuntimeError("missing current asyncio task")
    asyncio.get_running_loop().call_soon(current.cancel)
    try:
        await asyncio.Future()
    finally:
        _record("hold-finally:cancel")


def _ordered(labels):
    position = -1
    for label in labels:
        try:
            position = _events.index(label, position + 1)
        except ValueError:
            return False
    return True


async def stats():
    _record_loop()
    cancellation_ordered = _ordered(
        [
            "hold-start:cancel",
            "hold-finally:cancel",
            "aexit:cancel:CancelledError",
            "closed:cancel",
        ]
    )
    nested_lifo = _ordered(
        [
            "sync-enter:nested",
            "aenter:nested",
            "nested-body",
            "aexit:nested:None",
            "closed:nested",
            "sync-exit:nested",
        ]
    )
    return {
        "events": "|".join(_events),
        "loop_identity": next(iter(_loop_identities)) if len(_loop_identities) == 1 else "drift",
        "enter_count": _enter_count,
        "exit_count": _exit_count,
        "close_count": _close_count,
        "python_suppressed": _python_suppressed,
        "sifr_truthy_seen": _sifr_truthy_seen,
        "exit_failure_seen": _exit_failure_seen,
        "secondary_exit_failure_seen": _secondary_exit_failure_seen,
        "cancellation_ordered": cancellation_ordered,
        "nested_lifo": nested_lifo,
    }
