import pyarrow as pa
import pyarrow.compute as pc


def run() -> str:
    array = pa.array([1, 2, 3, 4])
    capsules = array.__arrow_c_array__()
    total = pc.sum(array).as_py()
    if total != 10 or len(capsules) != 2 or type(array).__module__ != "pyarrow.lib":
        raise RuntimeError("PyArrow full example did not validate expected Arrow results")
    return "sifr-python-interop:pyarrow:sum=10:kind=array:producer=pyarrow.lib"
