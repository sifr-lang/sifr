
# LeetCode 706: Design Hashmap
# Python version

from helpers.list_node import ListNode, list_node_to_string
class MyHashMap:
    def __init__(self):
        self.map = [ListNode() for i in range(1000)]
    def hashcode(self, key):
        return key % len(self.map)
    def put(self, key: int, value: int) -> None:
        cur = self.map[self.hashcode(key)]
        while cur.next:
            if cur.next.key == key:
                cur.next.val = value
                return
            cur = cur.next
        cur.next = ListNode(key, value)
    def get(self, key: int) -> int:
        cur = self.map[self.hashcode(key)].next
        while cur and cur.key != key:
            cur = cur.next
        if cur:
            return cur.val
        return -1
    def remove(self, key: int) -> None:
        cur = self.map[self.hashcode(key)]
        while cur.next and cur.next.key != key:
            cur = cur.next
        if cur and cur.next:
            cur.next = cur.next.next

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

if __name__ == "__main__":
    main()
