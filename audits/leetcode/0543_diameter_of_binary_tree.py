
# LeetCode 543: Diameter Of Binary Tree
# Python version
from helpers.tree_node import TreeNode, tree_to_string
def diameterOfBinaryTree(root: TreeNode | None) -> int:
    res = 0

    def dfs(root):
        nonlocal res

        if not root:
            return 0
        left = dfs(root.left)
        right = dfs(root.right)
        res = max(res, left + right)

        return 1 + max(left, right)

    dfs(root)
    return res

def main():
    assert diameterOfBinaryTree(TreeNode(1, TreeNode(2, TreeNode(4, None, None), TreeNode(5, None, None)), TreeNode(3, None, None))) == 3
    assert diameterOfBinaryTree(TreeNode(1, TreeNode(2, None, None), None)) == 1

if __name__ == "__main__":
    main()
