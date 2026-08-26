from __future__ import annotations

from importlib.metadata import version

import numpy as np
import pandas as pd


def main() -> int:
    if version("numpy") != "2.5.2":
        raise RuntimeError("NumPy is not at the audited stable release")
    if version("pandas") != "3.0.5":
        raise RuntimeError("Pandas is not at the audited stable release")

    values = np.array([3, 1, 2], dtype=np.int64)
    if np.sort(values, descending=True).tolist() != [3, 2, 1]:
        raise RuntimeError("NumPy descending sort behavior drifted")
    if np.argsort(values, descending=True).tolist() != [0, 2, 1]:
        raise RuntimeError("NumPy descending argsort behavior drifted")

    frame = pd.DataFrame({"city": ["oslo", None, "paris"], "value": [2, 3, 5]})
    if str(frame["city"].dtype) != "str" or not pd.isna(frame.iloc[1, 0]):
        raise RuntimeError("Pandas dedicated string dtype behavior drifted")

    cities = frame["city"]
    cities.iloc[0] = "bergen"
    if frame.iloc[0, 0] != "oslo":
        raise RuntimeError("Pandas copy-on-write behavior drifted")

    assigned = frame.assign(double=pd.col("value") * 2)
    if assigned["double"].tolist() != [4, 6, 10]:
        raise RuntimeError("Pandas column-expression behavior drifted")

    print(
        "python numeric/dataframe features ok: numpy=2.5.2 pandas=3.0.5 "
        "descending-sort=ok string-dtype=ok copy-on-write=ok pd-col=ok"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
