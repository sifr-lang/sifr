from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 1609: Even Odd Tree
# Python version

class TreeNode:
    def __init__(
        self,
        val: int = 0,
        left: 'TreeNode | None' = None,
        right: 'TreeNode | None' = None,
    ):
        self.val = val
        self.left = left
        self.right = right


def tree_to_string(node: TreeNode | None) -> str:
    if node is None:
        return "None"
    return f"{node.val}({tree_to_string(node.left)},{tree_to_string(node.right)})"


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

def isEvenOddTree(root: Optional[TreeNode]) -> bool:
    even = True
    q = deque([root])

    while q:
        prev = float("-inf") if even else float("inf")
        for _ in range(len(q)):
            node = q.popleft()

            if even and (node.val % 2 == 0 or node.val <= prev):
                return False
            elif not even and (node.val % 2 == 1 or node.val >= prev):
                return False
            
            if node.left:
                q.append(node.left)
            if node.right:
                q.append(node.right)
            prev = node.val
        even = not even
    return True



def main():
    assert isEvenOddTree(TreeNode(1, TreeNode(10, TreeNode(3, TreeNode(12, None, None), TreeNode(8, None, None)), None), TreeNode(4, TreeNode(7, TreeNode(6, None, None), None), TreeNode(9, None, TreeNode(2, None, None))))) == True
    assert isEvenOddTree(TreeNode(5, TreeNode(4, TreeNode(3, None, None), TreeNode(3, None, None)), TreeNode(2, TreeNode(7, None, None), None))) == False
    assert isEvenOddTree(TreeNode(5, TreeNode(9, TreeNode(3, None, None), TreeNode(5, None, None)), TreeNode(1, TreeNode(7, None, None), None))) == False

if __name__ == "__main__":
    main()