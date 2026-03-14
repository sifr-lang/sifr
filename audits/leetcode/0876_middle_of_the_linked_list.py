from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 876: Middle Of The Linked List
# Python version

class ListNode:
    def __init__(self, val: int = 0, next: 'ListNode | None' = None):
        self.val = val
        self.next = next


def list_node_to_string(node: ListNode | None) -> str:
    parts = []
    cur = node
    while cur is not None:
        parts.append(str(cur.val))
        cur = cur.next
    return "->".join(parts) if parts else "None"


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

def middleNode(head: Optional[ListNode]) -> Optional[ListNode]:
    if not head or not head.next:
        return head

    slow = fast = head
    while fast and fast.next:
        slow, fast = slow.next, fast.next.next

    return slow



def main():
    assert list_node_to_string(middleNode(ListNode(1, ListNode(2, ListNode(3, ListNode(4, ListNode(5, None))))))) == list_node_to_string(ListNode(3, ListNode(4, ListNode(5, None))))
    assert list_node_to_string(middleNode(ListNode(1, ListNode(2, ListNode(3, ListNode(4, ListNode(5, ListNode(6, None)))))))) == list_node_to_string(ListNode(4, ListNode(5, ListNode(6, None))))

if __name__ == "__main__":
    main()