# Concurrency Runtime CPython Evidence Matrix

Status: concurrency runtime inventory audited; generated from the capability source-of-truth list.

CPython checkout: `../cpython` at `14cbd0e6afa98355bdc6749b8230fed4c9b21bd6`.

| Reference | Domain | Native mapping | Evidence state | Extracted signal |
| --- | --- | --- | --- | --- |
| `Doc/library/asyncio-subprocess.rst` | subprocess/process | sifr.process | adapted-for-sifr-api | public_functions=29, keyword_forms=5, deprecation_markers=1 |
| `Doc/library/asyncio.rst` | queue/concurrency | sifr.task | adapted-for-sifr-api | keyword_forms=1 |
| `Doc/library/concurrent.futures.rst` | queue/concurrency | rules evidence | mined-as-substrate-fixture | public_functions=27, keyword_forms=12, deprecation_markers=3 |
| `Doc/library/contextlib.rst` | context/warnings/signal | sifr.resource | adapted-for-sifr-api | public_functions=31, keyword_forms=6, deprecation_markers=1 |
| `Doc/library/multiprocessing.rst` | queue/concurrency | sifr.ipc deferred worker substrate | rejected | public_functions=158, keyword_forms=28, deprecation_markers=3 |
| `Doc/library/multiprocessing.shared_memory.rst` | queue/concurrency | sifr.ipc deferred worker substrate | rejected | public_functions=16, keyword_forms=9 |
| `Doc/library/queue.rst` | queue/concurrency | sifr.sync | adapted-for-sifr-api | public_functions=28, keyword_forms=9 |
| `Doc/library/signal.rst` | context/warnings/signal | sifr.signal | adapted-for-sifr-api | public_functions=65, keyword_forms=4, deprecation_markers=1 |
| `Doc/library/subprocess.rst` | subprocess/process | sifr.process | adapted-for-sifr-api | public_functions=80, keyword_forms=40, deprecation_markers=2 |
| `Doc/library/warnings.rst` | context/warnings/signal | structured diagnostics | adapted-for-sifr-api | public_functions=20, keyword_forms=17, deprecation_markers=36 |
| `Lib/_py_warnings.py` | context/warnings/signal | structured diagnostics | adapted-for-sifr-api | public_functions=7, public_classes=3, keyword_forms=19, deprecation_markers=17 |
| `Lib/asyncio/__init__.py` | queue/concurrency | sifr.task | adapted-for-sifr-api |  |
| `Lib/asyncio/__main__.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | public_classes=2, public_methods=3 |
| `Lib/asyncio/base_events.py` | queue/concurrency | sifr.task | rejected | public_classes=2, public_methods=47, public_constants=1, keyword_forms=76 |
| `Lib/asyncio/base_futures.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | public_functions=1 |
| `Lib/asyncio/base_subprocess.py` | queue/concurrency | sifr.process | adapted-for-sifr-api | public_classes=3, public_methods=12 |
| `Lib/asyncio/base_tasks.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | deprecation_markers=1 |
| `Lib/asyncio/constants.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | public_constants=9 |
| `Lib/asyncio/coroutines.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | public_functions=2 |
| `Lib/asyncio/events.py` | queue/concurrency | sifr.task | rejected | public_functions=6, public_classes=4, public_methods=67, keyword_forms=77, deprecation_markers=1 |
| `Lib/asyncio/exceptions.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | public_classes=6 |
| `Lib/asyncio/format_helpers.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | public_functions=1, keyword_forms=2 |
| `Lib/asyncio/futures.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | public_functions=3, public_classes=1, public_methods=10, public_constants=1, keyword_forms=3, deprecation_markers=4 |
| `Lib/asyncio/graph.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | public_functions=3, public_classes=2, keyword_forms=7 |
| `Lib/asyncio/locks.py` | queue/concurrency | sifr.sync | adapted-for-sifr-api | public_classes=6, public_methods=21, keyword_forms=1 |
| `Lib/asyncio/log.py` | queue/concurrency | sifr.task | adapted-for-sifr-api |  |
| `Lib/asyncio/mixins.py` | queue/concurrency | sifr.task | adapted-for-sifr-api |  |
| `Lib/asyncio/proactor_events.py` | queue/concurrency | sifr.task | rejected | public_classes=1, public_methods=9, keyword_forms=1 |
| `Lib/asyncio/protocols.py` | queue/concurrency | sifr.task | rejected | public_classes=5, public_methods=14 |
| `Lib/asyncio/queues.py` | queue/concurrency | sifr.sync | adapted-for-sifr-api | public_classes=6, public_methods=11, keyword_forms=1, deprecation_markers=3 |
| `Lib/asyncio/runners.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | public_functions=1, public_classes=1, public_methods=3, keyword_forms=3 |
| `Lib/asyncio/selector_events.py` | queue/concurrency | sifr.task | rejected | public_classes=1, public_methods=13, keyword_forms=1 |
| `Lib/asyncio/sslproto.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | public_functions=1, public_classes=3, public_methods=7 |
| `Lib/asyncio/staggered.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | public_functions=1, keyword_forms=1 |
| `Lib/asyncio/streams.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | public_functions=2, public_classes=4, public_methods=28, keyword_forms=12, deprecation_markers=2 |
| `Lib/asyncio/subprocess.py` | queue/concurrency | sifr.process | adapted-for-sifr-api | public_functions=2, public_classes=2, public_methods=10, public_constants=3, keyword_forms=9 |
| `Lib/asyncio/taskgroups.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | public_classes=1, public_methods=1 |
| `Lib/asyncio/tasks.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | public_functions=12, public_classes=1, public_methods=11, public_constants=3, keyword_forms=12 |
| `Lib/asyncio/threads.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | public_functions=1 |
| `Lib/asyncio/timeouts.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | public_functions=2, public_classes=1, public_methods=3 |
| `Lib/asyncio/tools.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | public_functions=6, public_classes=2, keyword_forms=2 |
| `Lib/asyncio/transports.py` | queue/concurrency | sifr.task | rejected | public_classes=6, public_methods=24, keyword_forms=4 |
| `Lib/asyncio/trsock.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | public_classes=1, public_methods=15 |
| `Lib/asyncio/unix_events.py` | queue/concurrency | sifr.task | rejected | public_functions=2, deprecation_markers=1 |
| `Lib/asyncio/windows_events.py` | queue/concurrency | sifr.task | rejected | public_classes=3, public_methods=20, public_constants=6, keyword_forms=9 |
| `Lib/asyncio/windows_utils.py` | queue/concurrency | sifr.task | rejected | public_functions=1, public_classes=2, public_methods=3, public_constants=3, keyword_forms=4 |
| `Lib/concurrent/futures/__init__.py` | queue/concurrency | sifr.runtime / sifr.parallel | adapted-for-sifr-api |  |
| `Lib/concurrent/futures/_base.py` | queue/concurrency | sifr.runtime / sifr.parallel | adapted-for-sifr-api | public_functions=2, public_classes=6, public_methods=13, public_constants=9, keyword_forms=10, deprecation_markers=3 |
| `Lib/concurrent/futures/interpreter.py` | queue/concurrency | sifr.runtime / sifr.parallel | adapted-for-sifr-api | public_functions=1, public_classes=3, public_methods=5 |
| `Lib/concurrent/futures/process.py` | queue/concurrency | sifr.runtime / sifr.parallel | adapted-for-sifr-api | public_classes=2, public_methods=5, public_constants=1, keyword_forms=5, deprecation_markers=1 |
| `Lib/concurrent/futures/thread.py` | queue/concurrency | sifr.runtime / sifr.parallel | adapted-for-sifr-api | public_classes=3, public_methods=7, keyword_forms=2 |
| `Lib/contextlib.py` | context/warnings/signal | sifr.resource | adapted-for-sifr-api | public_functions=2, public_classes=13, public_methods=5, deprecation_markers=1 |
| `Lib/multiprocessing/__init__.py` | queue/concurrency | sifr.ipc deferred worker substrate | rejected | public_constants=2 |
| `Lib/multiprocessing/connection.py` | queue/concurrency | sifr.ipc deferred worker substrate | rejected | public_functions=7, public_classes=5, public_methods=9, public_constants=3, keyword_forms=3, deprecation_markers=14 |
| `Lib/multiprocessing/context.py` | queue/concurrency | sifr.ipc deferred worker substrate | rejected | public_functions=3, public_classes=7, public_methods=32, keyword_forms=22 |
| `Lib/multiprocessing/forkserver.py` | queue/concurrency | sifr.ipc deferred worker substrate | rejected | public_functions=3, public_classes=1, public_methods=4, public_constants=2, keyword_forms=6 |
| `Lib/multiprocessing/heap.py` | queue/concurrency | sifr.ipc deferred worker substrate | rejected | public_classes=2, public_methods=3 |
| `Lib/multiprocessing/managers.py` | queue/concurrency | sifr.ipc deferred worker substrate | rejected | public_functions=10, public_classes=21, public_methods=46, keyword_forms=23 |
| `Lib/multiprocessing/pool.py` | queue/concurrency | sifr.ipc deferred worker substrate | rejected | public_functions=4, public_classes=9, public_methods=18, public_constants=4, keyword_forms=23 |
| `Lib/multiprocessing/popen_fork.py` | queue/concurrency | sifr.ipc deferred worker substrate | rejected | public_classes=1, public_methods=7, keyword_forms=2 |
| `Lib/multiprocessing/popen_forkserver.py` | queue/concurrency | sifr.ipc deferred worker substrate | rejected | public_classes=1, public_methods=2, keyword_forms=1 |
| `Lib/multiprocessing/popen_spawn_posix.py` | queue/concurrency | sifr.ipc deferred worker substrate | rejected | public_classes=1, public_methods=1 |
| `Lib/multiprocessing/popen_spawn_win32.py` | queue/concurrency | sifr.ipc deferred worker substrate | rejected | public_classes=1, public_methods=5, public_constants=4, keyword_forms=1 |
| `Lib/multiprocessing/process.py` | queue/concurrency | sifr.ipc deferred worker substrate | rejected | public_functions=3, public_classes=2, public_methods=14, keyword_forms=1 |
| `Lib/multiprocessing/queues.py` | queue/concurrency | sifr.sync | rejected | public_classes=3, public_methods=17, keyword_forms=6, deprecation_markers=1 |
| `Lib/multiprocessing/reduction.py` | queue/concurrency | sifr.ipc deferred worker substrate | rejected | public_functions=1, public_classes=2, public_methods=2, public_constants=1, keyword_forms=2 |
| `Lib/multiprocessing/resource_sharer.py` | queue/concurrency | sifr.ipc deferred worker substrate | rejected |  |
| `Lib/multiprocessing/resource_tracker.py` | queue/concurrency | sifr.ipc deferred worker substrate | rejected | public_functions=2, public_classes=2, public_methods=4, deprecation_markers=1 |
| `Lib/multiprocessing/shared_memory.py` | queue/concurrency | sifr.ipc deferred worker substrate | rejected | public_classes=2, public_methods=8 |
| `Lib/multiprocessing/sharedctypes.py` | queue/concurrency | sifr.ipc deferred worker substrate | rejected | public_functions=9, public_classes=4, public_methods=2, keyword_forms=6 |
| `Lib/multiprocessing/spawn.py` | queue/concurrency | sifr.ipc deferred worker substrate | rejected | public_functions=9, keyword_forms=2 |
| `Lib/multiprocessing/synchronize.py` | queue/concurrency | sifr.ipc deferred worker substrate | rejected | public_classes=8, public_methods=10, public_constants=3, keyword_forms=4 |
| `Lib/multiprocessing/util.py` | queue/concurrency | sifr.ipc deferred worker substrate | rejected | public_functions=14, public_classes=3, public_methods=2, public_constants=8, keyword_forms=1, deprecation_markers=3 |
| `Lib/queue.py` | queue/concurrency | sifr.sync | adapted-for-sifr-api | public_classes=5, public_methods=10, keyword_forms=5, deprecation_markers=3 |
| `Lib/subprocess.py` | subprocess/process | sifr.process | adapted-for-sifr-api | public_functions=7, public_classes=5, public_methods=7, public_constants=3, keyword_forms=13, deprecation_markers=1 |
| `Lib/test/_test_multiprocessing.py` | queue/concurrency | sifr.ipc deferred worker substrate | mined-as-substrate-fixture | public_functions=22, public_classes=43, public_methods=114, public_constants=11, keyword_forms=10, test_classes=26, test_methods=79, deprecation_markers=6 |
| `Lib/test/test_asyncio/test_locks.py` | queue/concurrency | sifr.sync | adapted-for-sifr-api | public_functions=1, public_classes=5, public_methods=78, public_constants=2, test_classes=5, test_methods=75, deprecation_markers=1 |
| `Lib/test/test_asyncio/test_queues.py` | queue/concurrency | sifr.sync | adapted-for-sifr-api | public_functions=1, public_classes=11, public_methods=32, test_classes=11, test_methods=32, deprecation_markers=2 |
| `Lib/test/test_asyncio/test_runners.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | public_functions=2, public_classes=4, public_methods=36, test_classes=4, test_methods=30 |
| `Lib/test/test_asyncio/test_subprocess.py` | subprocess/process | sifr.process | adapted-for-sifr-api | public_functions=1, public_classes=3, public_methods=43, public_constants=2, keyword_forms=1, test_classes=2, test_methods=40 |
| `Lib/test/test_asyncio/test_taskgroups.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | public_functions=5, public_classes=5, public_methods=49, test_classes=3, test_methods=48 |
| `Lib/test/test_asyncio/test_tasks.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | public_functions=5, public_classes=26, public_methods=188, keyword_forms=10, test_classes=24, test_methods=167, deprecation_markers=3 |
| `Lib/test/test_asyncio/test_timeouts.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | public_functions=1, public_classes=1, public_methods=34, test_classes=1, test_methods=34 |
| `Lib/test/test_asyncio/test_waitfor.py` | queue/concurrency | sifr.task | adapted-for-sifr-api | public_functions=1, public_classes=3, public_methods=20, test_classes=2, test_methods=19 |
| `Lib/test/test_concurrent_futures/__init__.py` | queue/concurrency | rules evidence | mined-as-substrate-fixture | public_functions=1 |
| `Lib/test/test_concurrent_futures/executor.py` | queue/concurrency | rules evidence | mined-as-substrate-fixture | public_functions=4, public_classes=4, public_methods=20, keyword_forms=1, test_classes=1, test_methods=19 |
| `Lib/test/test_concurrent_futures/test_as_completed.py` | queue/concurrency | rules evidence | mined-as-substrate-fixture | public_functions=2, public_classes=1, public_methods=5, test_classes=1, test_methods=5 |
| `Lib/test/test_concurrent_futures/test_deadlock.py` | queue/concurrency | rules evidence | mined-as-substrate-fixture | public_functions=1, public_classes=7, public_methods=16, test_classes=1, test_methods=16 |
| `Lib/test/test_concurrent_futures/test_future.py` | queue/concurrency | rules evidence | mined-as-substrate-fixture | public_functions=1, public_classes=1, public_methods=21, test_classes=1, test_methods=21 |
| `Lib/test/test_concurrent_futures/test_init.py` | queue/concurrency | rules evidence | mined-as-substrate-fixture | public_functions=4, public_classes=3, public_methods=6, public_constants=1, keyword_forms=1, test_classes=1, test_methods=4 |
| `Lib/test/test_concurrent_futures/test_interpreter_pool.py` | queue/concurrency | rules evidence | mined-as-substrate-fixture | public_functions=9, public_classes=4, public_methods=32, public_constants=1, keyword_forms=2, test_classes=2, test_methods=28 |
| `Lib/test/test_concurrent_futures/test_process_pool.py` | queue/concurrency | rules evidence | mined-as-substrate-fixture | public_functions=1, public_classes=2, public_methods=20, public_constants=3, test_classes=1, test_methods=20 |
| `Lib/test/test_concurrent_futures/test_shutdown.py` | queue/concurrency | rules evidence | mined-as-substrate-fixture | public_functions=2, public_classes=3, public_methods=20, test_classes=3, test_methods=20 |
| `Lib/test/test_concurrent_futures/test_thread_pool.py` | queue/concurrency | rules evidence | mined-as-substrate-fixture | public_functions=1, public_classes=1, public_methods=7, test_classes=1, test_methods=7, deprecation_markers=1 |
| `Lib/test/test_concurrent_futures/test_wait.py` | queue/concurrency | rules evidence | mined-as-substrate-fixture | public_functions=3, public_classes=2, public_methods=9, test_classes=2, test_methods=9 |
| `Lib/test/test_concurrent_futures/util.py` | queue/concurrency | rules evidence | mined-as-substrate-fixture | public_functions=3, public_classes=7, public_methods=13, public_constants=6, keyword_forms=5, test_classes=1 |
| `Lib/test/test_contextlib.py` | context/warnings/signal | sifr.resource | adapted-for-sifr-api | public_classes=15, public_methods=90, test_classes=14, test_methods=88 |
| `Lib/test/test_contextlib_async.py` | context/warnings/signal | sifr.resource | adapted-for-sifr-api | public_classes=5, public_methods=39, test_classes=5, test_methods=39 |
| `Lib/test/test_io/test_signals.py` | context/warnings/signal | sifr.signal | adapted-for-sifr-api | public_classes=3, public_methods=16, test_classes=3, test_methods=9 |
| `Lib/test/test_multiprocessing_fork/__init__.py` | queue/concurrency | sifr.ipc deferred worker substrate | mined-as-substrate-fixture | public_functions=1 |
| `Lib/test/test_multiprocessing_fork/test_manager.py` | queue/concurrency | sifr.ipc deferred worker substrate | mined-as-substrate-fixture |  |
| `Lib/test/test_multiprocessing_fork/test_misc.py` | queue/concurrency | sifr.ipc deferred worker substrate | mined-as-substrate-fixture |  |
| `Lib/test/test_multiprocessing_fork/test_processes.py` | queue/concurrency | sifr.ipc deferred worker substrate | mined-as-substrate-fixture |  |
| `Lib/test/test_multiprocessing_fork/test_threads.py` | queue/concurrency | sifr.ipc deferred worker substrate | mined-as-substrate-fixture |  |
| `Lib/test/test_multiprocessing_forkserver/__init__.py` | queue/concurrency | sifr.ipc deferred worker substrate | mined-as-substrate-fixture | public_functions=1 |
| `Lib/test/test_multiprocessing_forkserver/test_manager.py` | queue/concurrency | sifr.ipc deferred worker substrate | mined-as-substrate-fixture |  |
| `Lib/test/test_multiprocessing_forkserver/test_misc.py` | queue/concurrency | sifr.ipc deferred worker substrate | mined-as-substrate-fixture |  |
| `Lib/test/test_multiprocessing_forkserver/test_preload.py` | queue/concurrency | sifr.ipc deferred worker substrate | mined-as-substrate-fixture | public_classes=2, public_methods=19, test_classes=2, test_methods=14 |
| `Lib/test/test_multiprocessing_forkserver/test_processes.py` | queue/concurrency | sifr.ipc deferred worker substrate | mined-as-substrate-fixture |  |
| `Lib/test/test_multiprocessing_forkserver/test_threads.py` | queue/concurrency | sifr.ipc deferred worker substrate | mined-as-substrate-fixture |  |
| `Lib/test/test_multiprocessing_main_handling.py` | queue/concurrency | sifr.ipc deferred worker substrate | mined-as-substrate-fixture | public_functions=1, public_classes=4, public_methods=14, public_constants=1, test_classes=3, test_methods=13 |
| `Lib/test/test_multiprocessing_spawn/__init__.py` | queue/concurrency | sifr.ipc deferred worker substrate | mined-as-substrate-fixture | public_functions=1 |
| `Lib/test/test_multiprocessing_spawn/test_manager.py` | queue/concurrency | sifr.ipc deferred worker substrate | mined-as-substrate-fixture |  |
| `Lib/test/test_multiprocessing_spawn/test_misc.py` | queue/concurrency | sifr.ipc deferred worker substrate | mined-as-substrate-fixture |  |
| `Lib/test/test_multiprocessing_spawn/test_processes.py` | queue/concurrency | sifr.ipc deferred worker substrate | mined-as-substrate-fixture |  |
| `Lib/test/test_multiprocessing_spawn/test_threads.py` | queue/concurrency | sifr.ipc deferred worker substrate | mined-as-substrate-fixture |  |
| `Lib/test/test_queue.py` | queue/concurrency | sifr.sync | adapted-for-sifr-api | public_functions=1, public_classes=18, public_methods=54, public_constants=1, test_classes=17, test_methods=35 |
| `Lib/test/test_signal.py` | context/warnings/signal | sifr.signal | adapted-for-sifr-api | public_functions=1, public_classes=12, public_methods=70, keyword_forms=2, test_classes=12, test_methods=57 |
| `Lib/test/test_subprocess.py` | subprocess/process | sifr.process | adapted-for-sifr-api | public_functions=1, public_classes=12, public_methods=265, public_constants=3, test_classes=10, test_methods=251, deprecation_markers=3 |
| `Lib/test/test_warnings/__init__.py` | context/warnings/signal | structured diagnostics | adapted-for-sifr-api | public_functions=2, public_classes=33, public_methods=108, keyword_forms=1, test_classes=33, test_methods=101, deprecation_markers=40 |
| `Lib/test/test_warnings/__main__.py` | context/warnings/signal | structured diagnostics | adapted-for-sifr-api |  |
| `Lib/test/test_warnings/data/import_warning.py` | context/warnings/signal | structured diagnostics | adapted-for-sifr-api | deprecation_markers=1 |
| `Lib/test/test_warnings/data/package_helper.py` | context/warnings/signal | structured diagnostics | adapted-for-sifr-api | public_functions=1, keyword_forms=2 |
| `Lib/test/test_warnings/data/stacklevel.py` | context/warnings/signal | structured diagnostics | adapted-for-sifr-api | public_functions=3, keyword_forms=5 |
| `Lib/warnings.py` | context/warnings/signal | structured diagnostics | adapted-for-sifr-api | deprecation_markers=2 |
| `Modules/_multiprocessing/multiprocessing.c` | queue/concurrency | sifr.ipc deferred worker substrate | mined-as-substrate-fixture | public_constants=6, keyword_forms=21 |
| `Modules/_multiprocessing/multiprocessing.h` | queue/concurrency | sifr.ipc deferred worker substrate | mined-as-substrate-fixture | public_constants=6, keyword_forms=1 |
| `Modules/_multiprocessing/posixshmem.c` | queue/concurrency | sifr.ipc deferred worker substrate | mined-as-substrate-fixture | public_constants=3, keyword_forms=15 |
| `Modules/_multiprocessing/semaphore.c` | queue/concurrency | sifr.ipc deferred worker substrate | mined-as-substrate-fixture | public_functions=4, public_constants=6, keyword_forms=40 |
| `Modules/_posixsubprocess.c` | subprocess/process | sifr.process | adapted-for-sifr-api | public_constants=25, keyword_forms=80, deprecation_markers=1 |
| `Modules/_queuemodule.c` | queue/concurrency | sifr.sync | adapted-for-sifr-api | keyword_forms=49 |
| `Modules/clinic/_posixsubprocess.c.h` | subprocess/process | sifr.process | adapted-for-sifr-api | public_functions=1, keyword_forms=25 |
| `Modules/clinic/_queuemodule.c.h` | queue/concurrency | sifr.sync | adapted-for-sifr-api | public_functions=8, keyword_forms=21 |
| `Modules/signalmodule.c` | context/warnings/signal | sifr.signal | adapted-for-sifr-api | public_functions=7, public_constants=92, keyword_forms=68 |
| `Python/_warnings.c` | context/warnings/signal | structured diagnostics | adapted-for-sifr-api | keyword_forms=80, deprecation_markers=1 |

## Notes

CPython module shapes are evidence only. Production Sifr APIs are native `sifr.*` surfaces, and CPython-shaped imports are rejected or diagnosed according to the inventory.
