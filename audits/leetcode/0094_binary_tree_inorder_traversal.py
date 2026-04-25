
# LeetCode 94: Binary Tree Inorder Traversal
# Python version
from helpers.tree_node import TreeNode, tree_to_string
def inorderTraversal(root: TreeNode | None) -> list[int]:
    # Iterative
    res, stack = [], []
    cur = root

    while cur or stack:
        while cur:
            stack.append(cur)
            cur = cur.left
        cur = stack.pop()
        res.append(cur.val)
        cur = cur.right
    return res

    # Recursive
    res = []

    def helper(root):
        if not root:
            return
        helper(root.left)
        res.append(root.val)
        helper(root.right)

    helper(root)
    return res

def main():
    assert inorderTraversal(TreeNode(1, None, TreeNode(2, TreeNode(3, None, None), None))) == [1, 3, 2]
    assert inorderTraversal(TreeNode(1, TreeNode(2, TreeNode(4, None, None), TreeNode(5, TreeNode(6, None, None), TreeNode(7, None, None))), TreeNode(3, None, TreeNode(8, TreeNode(9, None, None), None)))) == [4, 2, 6, 5, 7, 1, 3, 9, 8]
    assert inorderTraversal(None) == []

if __name__ == "__main__":
    main()
