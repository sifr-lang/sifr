from collections import deque

# LeetCode 104: Maximum Depth Of Binary Tree
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

def maxDepth(root: TreeNode) -> int:
    if not root:
        return 0

    return 1 + max(maxDepth(root.left), maxDepth(root.right))


# ITERATIVE DFS

def maxDepth(root: TreeNode) -> int:
    stack = [[root, 1]]
    res = 0

    while stack:
        node, depth = stack.pop()

        if node:
            res = max(res, depth)
            stack.append([node.left, depth + 1])
            stack.append([node.right, depth + 1])
    return res


# BFS

def maxDepth(root: TreeNode) -> int:
    q = deque()
    if root:
        q.append(root)

    level = 0

    while q:

        for i in range(len(q)):
            node = q.popleft()
            if node.left:
                q.append(node.left)
            if node.right:
                q.append(node.right)
        level += 1
    return level



def main():
    assert maxDepth(TreeNode(3, TreeNode(9, None, None), TreeNode(20, TreeNode(15, None, None), TreeNode(7, None, None)))) == 3
    assert maxDepth(TreeNode(1, None, TreeNode(2, None, None))) == 2

if __name__ == "__main__":
    main()