# LeetCode 707: Design Linked List
# Python version

class Node:
    def __init__(
        self,
        val: int = 0,
        next: 'Node | None' = None,
        random: 'Node | None' = None,
        left: 'Node | None' = None,
        right: 'Node | None' = None,
        neighbors: list['Node'] | None = None,
        key: int = -1,
    ):
        self.val = val
        self.next = next
        self.random = random
        self.left = left
        self.right = right
        self.neighbors = [] if neighbors is None else neighbors
        self.key = key

class ListNode:
    def __init__(self, val):
        self.val = val
        self.prev = None
        self.next = None
class MyLinkedList:
    def __init__(self):
        self.left = ListNode(0)
        self.right = ListNode(0)
        self.left.next = self.right
        self.right.prev = self.left
    def get(self, index: int) -> int:
        cur = self.left.next
        while cur and index > 0:
            cur = cur.next
            index -= 1
        if cur and cur != self.right and index == 0:
            return cur.val
        return -1
    def addAtHead(self, val: int) -> None:
        node, prev, next = ListNode(val), self.left, self.left.next
        node.next, node.prev = next, prev
        next.prev = node
        prev.next = node
    def addAtTail(self, val: int) -> None:
        node, prev, next = ListNode(val), self.right.prev, self.right
        node.next, node.prev = next, prev
        next.prev = node
        prev.next = node
    def addAtIndex(self, index: int, val: int) -> None:
        next = self.left.next
        while next and index > 0:
            next = next.next
            index -= 1
        if next and index == 0:
            node, prev = ListNode(val), next.prev
            node.next, node.prev = next, prev
            next.prev = node
            prev.next = node
    def deleteAtIndex(self, index: int) -> None:
        node = self.left.next
        while node and index > 0:
            node = node.next
            index -= 1
        if node and node != self.right and index == 0:
            node.prev.next = node.next
            node.next.prev = node.prev

def main():
    obj = MyLinkedList()
    obj.addAtHead(1)
    obj.addAtTail(3)
    obj.addAtIndex(1, 2)
    assert obj.get(1) == 2
    obj.deleteAtIndex(1)
    assert obj.get(1) == 3

if __name__ == "__main__":
    main()