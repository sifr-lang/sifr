# LeetCode 705: Design Hashset
# Python version

class MyHashSet:
    def __init__(self):
        self.hashset = []
    def add(self, key: int) -> None:
        if not self.contains(key):
            self.hashset.append(key)
    def remove(self, key: int) -> None:
        if self.contains(key):
            self.hashset.remove(key)
    def contains(self, key: int) -> bool:
        return True if key in self.hashset else False

def main():
    obj = MyHashSet()
    obj.add(1)
    obj.add(2)
    assert obj.contains(1) == True
    assert obj.contains(3) == False
    obj.add(2)
    assert obj.contains(2) == True
    obj.remove(2)
    assert obj.contains(2) == False

if __name__ == "__main__":
    main()
