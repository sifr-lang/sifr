from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 662: Maximum Width Of Binary Tree
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

def widthOfBinaryTree(root: Optional[TreeNode]) -> int:
    if root is None:
        return 0

    q = [(root, 0)]
    width = 0
    while q:
        leftIndex = q[0][1]
        rightIndex = q[-1][1]
        width = max(width, rightIndex - leftIndex + 1)

        for _ in range(len(q)):
            node, index = q.pop(0)
            if node.left:
                q.append((node.left, index * 2))
            if node.right:
                q.append((node.right, index * 2 + 1))
    return width



def main():
    assert widthOfBinaryTree(TreeNode(1, TreeNode(3, TreeNode(5, None, None), TreeNode(3, None, None)), TreeNode(2, None, TreeNode(9, None, None)))) == 4
    assert widthOfBinaryTree(TreeNode(1, TreeNode(3, TreeNode(5, TreeNode(6, None, None), None), None), TreeNode(2, None, TreeNode(9, TreeNode(7, None, None), None)))) == 7
    assert widthOfBinaryTree(TreeNode(1, TreeNode(3, TreeNode(5, None, None), None), TreeNode(2, None, None))) == 2

if __name__ == "__main__":
    main()