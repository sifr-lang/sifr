import pyarrow as pa
import pyarrow.compute as pc


def run() -> str:
    array = pa.array([1, 2, 3, 4])
    capsules = array.__arrow_c_array__()
    capsule_names = tuple(repr(capsule) for capsule in capsules)
    total = pc.sum(array).as_py()
    hypotenuse = pc.hypot(pa.scalar(3.0), pa.scalar(4.0)).as_py()
    tensor = pa.table({"x": [1, 2], "y": [3, 4]}).to_tensor()
    if (
        pa.__version__ != "25.0.1"
        or total != 10
        or hypotenuse != 5.0
        or tensor.shape != (2, 2)
        or tensor.to_numpy().tolist() != [[1, 3], [2, 4]]
        or len(capsules) != 2
        or '"arrow_schema"' not in capsule_names[0]
        or '"arrow_array"' not in capsule_names[1]
        or type(array).__module__ != "pyarrow.lib"
    ):
        raise RuntimeError(
            "PyArrow full example did not validate expected Arrow results"
        )
    return (
        "sifr-python-interop:pyarrow:sum=10:hypot=5:table-tensor=2x2:"
        "kind=array:producer=pyarrow.lib"
    )
