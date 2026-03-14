from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 24: Swap Nodes In Pairs
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

def swapPairs(head: ListNode) -> ListNode:
    dummy = ListNode(0, head)
    prev, curr = dummy, head

    while curr and curr.next:
        # save ptrs
        nxtPair = curr.next.next
        second = curr.next

        # reverse this pair
        second.next = curr
        curr.next = nxtPair
        prev.next = second

        # update ptrs
        prev = curr
        curr = nxtPair

    return dummy.next



def main():
    assert list_node_to_string(swapPairs(ListNode(1, ListNode(2, ListNode(3, ListNode(4, None)))))) == list_node_to_string(ListNode(2, ListNode(1, ListNode(4, ListNode(3, None)))))
    assert swapPairs(None) == None
    assert list_node_to_string(swapPairs(ListNode(1, None))) == list_node_to_string(ListNode(1, None))

if __name__ == "__main__":
    main()