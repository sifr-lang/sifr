from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 1721: Swapping Nodes In A Linked List
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

def swapNodes(head: Optional[ListNode], k: int) -> Optional[ListNode]:
    right_pointer = head
    for _ in range(1, k):
        right_pointer = right_pointer.next
    left_kth_node = right_pointer

    left_pointer = head
    while right_pointer is not None:
        right_kth_node = left_pointer
        right_pointer = right_pointer.next
        left_pointer = left_pointer.next

    left_kth_node.val, right_kth_node.val = right_kth_node.val, left_kth_node.val
    return head



def main():
    assert list_node_to_string(swapNodes(ListNode(1, ListNode(2, ListNode(3, ListNode(4, ListNode(5, None))))), 2)) == list_node_to_string(ListNode(1, ListNode(4, ListNode(3, ListNode(2, ListNode(5, None))))))
    assert list_node_to_string(swapNodes(ListNode(7, ListNode(9, ListNode(6, ListNode(6, ListNode(7, ListNode(8, ListNode(3, ListNode(0, ListNode(9, ListNode(5, None)))))))))), 5)) == list_node_to_string(ListNode(7, ListNode(9, ListNode(6, ListNode(6, ListNode(8, ListNode(7, ListNode(3, ListNode(0, ListNode(9, ListNode(5, None)))))))))))

if __name__ == "__main__":
    main()