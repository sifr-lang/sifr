import json


def same_prefix(value: str) -> bool:
    return value == "sifr" or value == "python"


def main() -> None:
    values = [
        True and not False,
        same_prefix("sifr"),
        same_prefix("rust"),
        len("safe") == 4,
        "si" + "fr" == "sifr",
    ]
    print(json.dumps(values, separators=(",", ":")))


if __name__ == "__main__":
    main()
