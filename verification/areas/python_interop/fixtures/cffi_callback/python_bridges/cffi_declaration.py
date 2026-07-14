import cffi
import threading


def apply_current(handler, value):
    ffi = cffi.FFI()
    callback = ffi.callback("long long(long long)", handler)
    return callback(value)


def apply(handler, value):
    ffi = cffi.FFI()
    callback = ffi.callback("long long(long long)", handler)
    results = []
    worker = threading.Thread(
        target=lambda: results.append(callback(value)),
        name="sifr-cffi-callback",
    )
    worker.start()
    worker.join()
    return results[0]
