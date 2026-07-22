import polars as pl


def run() -> str:
    frame = pl.DataFrame({"city": ["paris", "oslo", "rome"], "value": [5, 2, 3]})
    sorted_frame = frame.sort("value")
    values = sorted_frame["value"].to_list()
    cities = sorted_frame["city"].to_list()
    if values != [2, 3, 5] or cities[0] != "oslo" or int(sum(values)) != 10:
        raise RuntimeError("Polars full example did not round-trip expected values")
    return "sifr-python-interop:polars:sum=10:first-city=oslo"
