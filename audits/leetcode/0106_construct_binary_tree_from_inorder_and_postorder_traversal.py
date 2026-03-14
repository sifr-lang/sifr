from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 106: Construct Binary Tree From Inorder And Postorder Traversal
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

def buildTree(inorder: List[int], postorder: List[int]) -> Optional[TreeNode]:
    def buildTreeHelper(left, right):
        if left > right:
            return None

        rootVal = postorder.pop()
        rootNode = TreeNode(rootVal)

        idx = inorderIndexMap[rootVal]
        rootNode.right = buildTreeHelper(idx + 1, right)
        rootNode.left = buildTreeHelper(left, idx - 1)
        return rootNode

    inorderIndexMap = {}
    for (i, val) in enumerate(inorder):
        inorderIndexMap[val] = i

    return buildTreeHelper(0, len(postorder) - 1)



def main():
    assert tree_to_string(buildTree([9, 3, 15, 20, 7], [9, 15, 7, 20, 3])) == "3(9(None,None),20(15(None,None),7(None,None)))"
    assert tree_to_string(buildTree([-1], [-1])) == "-1(None,None)"

if __name__ == "__main__":
    main()
