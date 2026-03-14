# LeetCode 380: Insert Delete Getrandom O1
# Python version

class RandomizedSet:
    def __init__(self):
        self.dict = {}
        self.list = []
    def insert(self, val: int) -> bool:
        if val in self.dict:
            return False
        self.dict[val] = len(self.list)
        self.list.append(val)
        return True
    def remove(self, val: int) -> bool:
        if val not in self.dict:
            return False
        idx, last_element = self.dict[val], self.list[-1]
        self.list[idx], self.dict[last_element] = last_element, idx
        self.list.pop()
        del self.dict[val]
        return True
    def getRandom(self) -> int:
        return choice(self.list)

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
