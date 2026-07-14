import cffi


def apply(handler, value):
    ffi = cffi.FFI()
    callback = ffi.callback("long long(long long)", handler)
    return callback(value)
