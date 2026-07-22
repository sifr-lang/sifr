import pandas as pd


def run() -> str:
    frame = pd.DataFrame({"city": ["oslo", "oslo", "paris"], "value": [2, 3, 5]})
    values = frame["value"].tolist()
    grouped = frame.assign(double=frame["value"].mul(2)).groupby("city").sum()
    if values != [2, 3, 5] or int(grouped["double"].sum()) != 20:
        raise RuntimeError("pandas full example did not round-trip expected values")
    return "sifr-python-interop:pandas:double-total=20:values=2,3,5"
