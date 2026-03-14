from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 226: Invert Binary Tree
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

def invertTree(root: Optional[TreeNode]) -> Optional[TreeNode]:
    if not root:
        return None
    
    # swap the children
    root.left, root.right = root.right, root.left
    
    # make 2 recursive calls
    invertTree(root.left)
    invertTree(root.right)
    return root



def main():
    assert tree_to_string(invertTree(TreeNode(4, TreeNode(2, TreeNode(1, None, None), TreeNode(3, None, None)), TreeNode(7, TreeNode(6, None, None), TreeNode(9, None, None))))) == tree_to_string(TreeNode(4, TreeNode(7, TreeNode(9, None, None), TreeNode(6, None, None)), TreeNode(2, TreeNode(3, None, None), TreeNode(1, None, None))))
    assert tree_to_string(invertTree(TreeNode(2, TreeNode(1, None, None), TreeNode(3, None, None)))) == tree_to_string(TreeNode(2, TreeNode(3, None, None), TreeNode(1, None, None)))
    assert invertTree(None) == None

if __name__ == "__main__":
    main()