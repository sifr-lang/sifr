
# LeetCode 124: Binary Tree Maximum Path Sum
# Python version
from helpers.tree_node import TreeNode, tree_to_string
def maxPathSum(root: TreeNode) -> int:
    res = [root.val]

    # return max path sum without split
    def dfs(root):
        if not root:
            return 0

        leftMax = dfs(root.left)
        rightMax = dfs(root.right)
        leftMax = max(leftMax, 0)
        rightMax = max(rightMax, 0)

        # compute max path sum WITH split
        res[0] = max(res[0], root.val + leftMax + rightMax)
        return root.val + max(leftMax, rightMax)

    dfs(root)
    return res[0]

def main():
    assert maxPathSum(TreeNode(1, TreeNode(2, None, None), TreeNode(3, None, None))) == 6
    assert maxPathSum(TreeNode(-10, TreeNode(9, None, None), TreeNode(20, TreeNode(15, None, None), TreeNode(7, None, None)))) == 42

if __name__ == "__main__":
    main()
