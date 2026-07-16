import numpy


_last_numpy = None


def make_numpy_range(stop: int):
    global _last_numpy
    _last_numpy = numpy.arange(stop, dtype=numpy.int64)
    return _last_numpy


def last_numpy_pointer() -> int:
    if _last_numpy is None:
        return 0
    return int(_last_numpy.__array_interface__["data"][0])


def last_numpy_value(index: int) -> int:
    if _last_numpy is None:
        raise RuntimeError("NumPy producer has not run")
    return int(_last_numpy[index])
