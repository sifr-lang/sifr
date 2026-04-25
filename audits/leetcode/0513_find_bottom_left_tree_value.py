from helpers.tree_node import TreeNode, tree_to_string

from collections import deque

# LeetCode 513: Find Bottom Left Tree Value
# Python version
def findBottomLeftValue(root: TreeNode | None) -> int:

    res = []
    q = deque()
    q.append(root)
    while q:
        qlen = len(q)
        level = []
        for i in range(qlen):
            node = q.popleft()
            if node:
                q.append(node.left)
                q.append(node.right)
                level.append(node.val)
        if level:
            res.append(level)
    return res[-1][0]
  
# recursive

def findBottomLeftValue(root: TreeNode | None) -> int:
    max_height = -1
    res = -1
    def dfs(root, depth):
        nonlocal max_height, res
        if not root:
            return
        if depth > max_height:
            max_height = max(depth, max_height)
            res = root.val
        dfs(root.left, depth + 1)
        dfs(root.right, depth + 1)
    
    dfs(root, 0)
    return res

def main():
    assert findBottomLeftValue(TreeNode(2, TreeNode(1, None, None), TreeNode(3, None, None))) == 1
    assert findBottomLeftValue(TreeNode(1, TreeNode(2, TreeNode(4, None, None), None), TreeNode(3, TreeNode(5, TreeNode(7, None, None), None), TreeNode(6, None, None)))) == 7

if __name__ == "__main__":
    main()
