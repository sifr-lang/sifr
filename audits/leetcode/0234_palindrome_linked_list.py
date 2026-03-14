from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 234: Palindrome Linked List
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

def isPalindrome(head: ListNode) -> bool:
    fast = head
    slow = head
    
    # find the middle (slow)
    while fast and fast.next:
        fast = fast.next.next
        slow = slow.next
        
    # reverse second half
    prev = None
    while slow:
        tmp = slow.next
        slow.next = prev
        prev = slow
        slow = tmp
    
    # check palindrome
    left, right = head, prev
    while right:
        if left.val != right.val:
            return False
        left = left.next
        right = right.next
    return True



def main():
    assert isPalindrome(ListNode(1, ListNode(2, ListNode(2, ListNode(1, None))))) == True
    assert isPalindrome(ListNode(1, ListNode(2, None))) == False

if __name__ == "__main__":
    main()