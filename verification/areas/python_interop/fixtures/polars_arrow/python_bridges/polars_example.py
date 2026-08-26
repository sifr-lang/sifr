import polars as pl


def run() -> str:
    frame = pl.DataFrame({"city": ["paris", "oslo", "rome"], "value": [5, 2, 3]})
    sorted_frame = frame.sort("value")
    city_rows = sorted_frame.with_columns(
        pl.struct("city", "value").alias("row")
    ).select(pl.col("row").struct.drop(["value"]))
    values = sorted_frame["value"].to_list()
    cities = sorted_frame["city"].to_list()
    projected_cities = city_rows["row"].struct.field("city").to_list()
    if (
        values != [2, 3, 5]
        or cities[0] != "oslo"
        or projected_cities != cities
        or int(sum(values)) != 10
    ):
        raise RuntimeError("Polars full example did not round-trip expected values")
    return "sifr-python-interop:polars:sum=10:first-city=oslo:struct-drop=ok"
