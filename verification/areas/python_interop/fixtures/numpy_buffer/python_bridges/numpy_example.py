import numpy as np


def run() -> str:
    values = np.array([1, 2, 3, 4], dtype="int64").reshape(2, 2)
    doubled = np.multiply(values, 2)
    copied = doubled.ravel().tolist()
    if copied != [2, 4, 6, 8] or int(doubled.sum()) != 20:
        raise RuntimeError("NumPy full example did not round-trip expected values")
    return "sifr-python-interop:numpy:sum=20:values=2,4,6,8"
