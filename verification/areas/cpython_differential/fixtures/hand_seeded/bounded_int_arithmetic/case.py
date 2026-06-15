import json


def main() -> None:
    values = [1 + 2 * 3, 17 // 5, 17 % 5, -5 + 9, (8 - 3) * 4]
    print(json.dumps(values, separators=(",", ":")))


if __name__ == "__main__":
    main()
