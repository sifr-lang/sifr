from helpers.tree_node import TreeNode, tree_to_string

import collections

# LeetCode 102: Binary Tree Level Order Traversal
# Python version
def levelOrder(root: TreeNode) -> list[list[int]]:
    res = []
    q = collections.deque()
    if root:
        q.append(root)

    while q:
        val = []

        for i in range(len(q)):
            node = q.popleft()
            val.append(node.val)
            if node.left:
                q.append(node.left)
            if node.right:
                q.append(node.right)
        res.append(val)
    return res

def main():
    assert levelOrder(TreeNode(3, TreeNode(9, None, None), TreeNode(20, TreeNode(15, None, None), TreeNode(7, None, None)))) == [[3], [9, 20], [15, 7]]
    assert levelOrder(TreeNode(1, None, None)) == [[1]]
    assert levelOrder(None) == []

if __name__ == "__main__":
    main()
