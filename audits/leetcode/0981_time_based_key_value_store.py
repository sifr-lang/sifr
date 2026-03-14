
# LeetCode 981: Time Based Key Value Store
# Python version

class TimeMap:
    def __init__(self):
        """
        Initialize your data structure here.
        """
        self.keyStore = {}  # key : list of [val, timestamp]
    def set(self, key: str, value: str, timestamp: int) -> None:
        if key not in self.keyStore:
            self.keyStore[key] = []
        self.keyStore[key].append([value, timestamp])
    def get(self, key: str, timestamp: int) -> str:
        res, values = "", self.keyStore.get(key, [])
        l, r = 0, len(values) - 1
        while l <= r:
            m = (l + r) // 2
            if values[m][1] <= timestamp:
                res = values[m][0]
                l = m + 1
            else:
                r = m - 1
        return res

def main():
    obj = TimeMap()
    obj.set('foo', 'bar', 1)
    assert obj.get('foo', 1) == 'bar'
    assert obj.get('foo', 3) == 'bar'
    obj.set('foo', 'bar2', 4)
    assert obj.get('foo', 4) == 'bar2'
    assert obj.get('foo', 5) == 'bar2'

if __name__ == "__main__":
    main()
