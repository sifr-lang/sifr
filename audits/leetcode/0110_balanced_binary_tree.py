
# LeetCode 110: Balanced Binary Tree
# Python version
from helpers.tree_node import TreeNode, tree_to_string
def isBalanced(root: TreeNode | None) -> bool:
    def dfs(root):
        if not root:
            return [True, 0]

        left, right = dfs(root.left), dfs(root.right)
        balanced = left[0] and right[0] and abs(left[1] - right[1]) <= 1
        return [balanced, 1 + max(left[1], right[1])]

    return dfs(root)[0]

def main():
    assert isBalanced(TreeNode(3, TreeNode(9, None, None), TreeNode(20, TreeNode(15, None, None), TreeNode(7, None, None)))) == True
    assert isBalanced(TreeNode(1, TreeNode(2, TreeNode(3, TreeNode(4, None, None), TreeNode(4, None, None)), TreeNode(3, None, None)), TreeNode(2, None, None))) == False
    assert isBalanced(None) == True

if __name__ == "__main__":
    main()
