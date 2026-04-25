
# LeetCode 669: Trim A Binary Search Tree
# Python version
from helpers.tree_node import TreeNode, tree_to_string
def trimBST(root: TreeNode | None, low: int, high: int) -> TreeNode | None:
    if not root:
        return None

    if root.val > high:
        return trimBST(root.left, low, high)

    if root.val < low:
        return trimBST(root.right, low, high)

    else:
        root.left = trimBST(root.left, low, high)
        root.right = trimBST(root.right, low, high)
        return root

def main():
    assert tree_to_string(trimBST(TreeNode(1, TreeNode(0, None, None), TreeNode(2, None, None)), 1, 2)) == tree_to_string(TreeNode(1, None, TreeNode(2, None, None)))
    assert tree_to_string(trimBST(TreeNode(3, TreeNode(0, None, TreeNode(2, TreeNode(1, None, None), None)), TreeNode(4, None, None)), 1, 3)) == tree_to_string(TreeNode(3, TreeNode(2, TreeNode(1, None, None), None), None))

if __name__ == "__main__":
    main()
