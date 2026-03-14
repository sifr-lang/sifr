from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 235: Lowest Common Ancestor Of A Binary Search Tree
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

def lowestCommonAncestor(
    self, root: "TreeNode", p: "TreeNode", q: "TreeNode"
) -> "TreeNode":
    while True:
        if root.val < p.val and root.val < q.val:
            root = root.right
        elif root.val > p.val and root.val > q.val:
            root = root.left
        else:
            return root



def main():
    root = TreeNode(
        6,
        TreeNode(2, TreeNode(0), TreeNode(4, TreeNode(3), TreeNode(5))),
        TreeNode(8, TreeNode(7), TreeNode(9)),
    )
    assert tree_to_string(
        lowestCommonAncestor(None, root, root.left, root.right)
    ) == tree_to_string(root)
    assert tree_to_string(
        lowestCommonAncestor(None, root, root.left, root.left.right)
    ) == tree_to_string(root.left)
    assert tree_to_string(
        lowestCommonAncestor(None, TreeNode(2, TreeNode(1), None), TreeNode(2), TreeNode(1))
    ) == tree_to_string(TreeNode(2, TreeNode(1), None))

if __name__ == "__main__":
    main()
