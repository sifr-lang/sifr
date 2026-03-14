from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 572: Subtree Of Another Tree
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

def isSubtree(root: Optional[TreeNode], subRoot: Optional[TreeNode]) -> bool:
    if not subRoot:
        return True
    if not root:
        return False

    if isSameTree(root, subRoot):
        return True
    return isSubtree(root.left, subRoot) or isSubtree(root.right, subRoot)


def isSameTree(p: Optional[TreeNode], q: Optional[TreeNode]) -> bool:
    if not p and not q:
        return True
    if p and q and p.val == q.val:
        return isSameTree(p.left, q.left) and isSameTree(p.right, q.right)
    else:
        return False



def main():
    assert isSubtree(TreeNode(3, TreeNode(4, TreeNode(1, None, None), TreeNode(2, None, None)), TreeNode(5, None, None)), TreeNode(4, TreeNode(1, None, None), TreeNode(2, None, None))) == True
    assert isSubtree(TreeNode(3, TreeNode(4, TreeNode(1, None, None), TreeNode(2, TreeNode(0, None, None), None)), TreeNode(5, None, None)), TreeNode(4, TreeNode(1, None, None), TreeNode(2, None, None))) == False

if __name__ == "__main__":
    main()