
# LeetCode 706: Design Hashmap
# Python version

BUCKET_COUNT = 769


class MyHashMap:
    def __init__(self):
        self.buckets: list[list[tuple[int, int]]] = [[] for _ in range(BUCKET_COUNT)]

    def hashcode(self, key: int) -> int:
        return key % len(self.buckets)

    def put(self, key: int, value: int) -> None:
        bucket = self.buckets[self.hashcode(key)]
        for index, (existing_key, _existing_value) in enumerate(bucket):
            if existing_key == key:
                bucket[index] = (key, value)
                return
        bucket.append((key, value))

    def get(self, key: int) -> int:
        bucket = self.buckets[self.hashcode(key)]
        for existing_key, existing_value in bucket:
            if existing_key == key:
                return existing_value
        return -1

    def remove(self, key: int) -> None:
        bucket = self.buckets[self.hashcode(key)]
        self.buckets[self.hashcode(key)] = [
            (existing_key, existing_value)
            for existing_key, existing_value in bucket
            if existing_key != key
        ]


def main():
    obj = MyHashMap()
    obj.put(1, 1)
    obj.put(2, 2)
    assert obj.get(1) == 1
    assert obj.get(3) == -1
    obj.put(2, 1)
    assert obj.get(2) == 1
    obj.remove(2)
    assert obj.get(2) == -1
    obj.put(5, -1)
    assert obj.get(5) == -1
    obj.put(5, 7)
    assert obj.get(5) == 7
    obj.remove(5)
    assert obj.get(5) == -1

if __name__ == "__main__":
    main()
