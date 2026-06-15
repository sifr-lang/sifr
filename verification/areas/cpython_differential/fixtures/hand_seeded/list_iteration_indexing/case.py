import json


def main() -> None:
    items = [3, 1, 4, 1, 5]
    total = 0
    for value in items:
        total = total + value
    values = [len(items) == 5, items[0] == 3, items[2] == 4, total == 14]
    print(json.dumps(values, separators=(",", ":")))


if __name__ == "__main__":
    main()
