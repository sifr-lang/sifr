
# LeetCode 707: Design Linked List
# Python version
from helpers.list_node import ListNode, list_node_to_string

class MyLinkedList:
    def __init__(self):
        self.left = ListNode(0)
        self.right = ListNode(0)
        self.left.next = self.right
        self.right.prev = self.left
        self.size = 0

    def get(self, index: int) -> int:
        if index < 0 or index >= self.size:
            return -1
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
        self.size += 1

    def addAtTail(self, val: int) -> None:
        node, prev, next = ListNode(val), self.right.prev, self.right
        node.next, node.prev = next, prev
        next.prev = node
        prev.next = node
        self.size += 1

    def addAtIndex(self, index: int, val: int) -> None:
        if index < 0:
            index = 0
        if index > self.size:
            return
        next = self.left.next
        while next and index > 0:
            next = next.next
            index -= 1
        if next and index == 0:
            node, prev = ListNode(val), next.prev
            node.next, node.prev = next, prev
            next.prev = node
            prev.next = node
            self.size += 1

    def deleteAtIndex(self, index: int) -> None:
        if index < 0 or index >= self.size:
            return
        node = self.left.next
        while node and index > 0:
            node = node.next
            index -= 1
        if node and node != self.right and index == 0:
            node.prev.next = node.next
            node.next.prev = node.prev
            self.size -= 1

def main():
    obj = MyLinkedList()
    obj.addAtHead(1)
    obj.addAtTail(3)
    obj.addAtIndex(1, 2)
    assert obj.get(1) == 2
    obj.deleteAtIndex(1)
    assert obj.get(1) == 3
    obj.addAtIndex(3, 4)
    assert obj.get(3) == -1
    obj.addAtIndex(-1, 5)
    assert obj.get(0) == 5
    obj.deleteAtIndex(0)
    assert obj.get(0) == 1

if __name__ == "__main__":
    main()
