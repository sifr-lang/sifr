
# LeetCode 380: Insert Delete Getrandom O1
# Python version

def choice(values: list[int]) -> int:
    return values[0]


def last_value(values: list[int]) -> int:
    return values[-1]


class RandomizedSet:
    def __init__(self):
        self.index_map: dict[int, int] = {}
        self.values: list[int] = []

    def insert(self, val: int) -> bool:
        if self.index_map.get(val, -1) != -1:
            return False
        self.index_map[val] = len(self.values)
        self.values.append(val)
        return True

    def remove(self, val: int) -> bool:
        idx = self.index_map.get(val, -1)
        if idx == -1:
            return False
        last_element = last_value(self.values)
        self.values[idx] = last_element
        self.index_map[last_element] = idx
        self.values.pop()
        self.index_map[val] = -1
        return True

    def getRandom(self) -> int:
        return choice(self.values)


def main():
    randomized_set = RandomizedSet()
    assert randomized_set.insert(1) is True
    assert randomized_set.remove(2) is False
    assert randomized_set.insert(2) is True
    assert randomized_set.getRandom() == 1
    assert randomized_set.remove(1) is True
    assert randomized_set.insert(2) is False
    assert randomized_set.getRandom() == 2


if __name__ == "__main__":
    main()
