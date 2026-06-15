import json


def main() -> None:
    payload = {"first": 2, "second": 3, "third": 5}
    values = [len(payload) == 3, payload["first"] == 2, payload["third"] == 5]
    print(json.dumps(values, separators=(",", ":")))


if __name__ == "__main__":
    main()
