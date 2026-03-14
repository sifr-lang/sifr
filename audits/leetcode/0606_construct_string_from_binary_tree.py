from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 606: Construct String From Binary Tree
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

def tree2str(root: Optional[TreeNode]) -> str:
    # Solution with O(n) time and space complexity
    res = []
    dfs(root, res)
    return "".join(res)


def dfs(t: TreeNode, res: list):
    # If the current node is None, do nothing and return
    if t is None:
        return
    res.append(str(t.val))

    # If both left and right children are None, return as there are no more branches to explore
    if t.left is None and t.right is None:
        return
    res.append('(')

    # Recursively call the DFS function for the left child
    dfs(t.left, res)
    res.append(')')

    # If the right child exists, process it
    if t.right is not None:
        res.append('(')

        # Recursively call the DFS function for the right child
        dfs(t.right, res)
        res.append(')')
        


def main():
    assert tree2str(TreeNode(1, TreeNode(2, TreeNode(4, None, None), None), TreeNode(3, None, None))) == '1(2(4))(3)'
    assert tree2str(TreeNode(1, TreeNode(2, None, TreeNode(4, None, None)), TreeNode(3, None, None))) == '1(2()(4))(3)'

if __name__ == "__main__":
    main()