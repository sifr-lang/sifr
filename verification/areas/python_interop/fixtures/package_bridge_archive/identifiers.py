import biip


def parse_gtin(text: str) -> dict[str, object]:
    gtin = biip.parse(text).gtin
    return {
        "value": gtin.value,
        "format": gtin.format.value,
        "check_digit": gtin.check_digit,
    }
