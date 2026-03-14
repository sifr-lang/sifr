from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 669: Trim A Binary Search Tree
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

def trimBST(root: Optional[TreeNode], low: int, high: int) -> Optional[TreeNode]:
    if not root:
        return None

    if root.val > high:
        return trimBST(root.left, low, high)

    if root.val < low:
        return trimBST(root.right, low, high)

    else:
        root.left = trimBST(root.left, low, high)
        root.right = trimBST(root.right, low, high)
        return root


def main():
    assert tree_to_string(trimBST(TreeNode(1, TreeNode(0, None, None), TreeNode(2, None, None)), 1, 2)) == tree_to_string(TreeNode(1, None, TreeNode(2, None, None)))
    assert tree_to_string(trimBST(TreeNode(3, TreeNode(0, None, TreeNode(2, TreeNode(1, None, None), None)), TreeNode(4, None, None)), 1, 3)) == tree_to_string(TreeNode(3, TreeNode(2, TreeNode(1, None, None), None), None))

if __name__ == "__main__":
    main()