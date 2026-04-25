
# LeetCode 701: Insert Into A Binary Search Tree
# Python version
from helpers.tree_node import TreeNode, tree_to_string
def insertIntoBST(root: TreeNode | None, val: int) -> TreeNode | None:
    if not root:
        return TreeNode(val)
    if val > root.val:
        root.right = insertIntoBST(root.right, val)
    else:
        root.left = insertIntoBST(root.left, val)
    return root

def main():
    assert tree_to_string(insertIntoBST(TreeNode(4, TreeNode(2, TreeNode(1, None, None), TreeNode(3, None, None)), TreeNode(7, None, None)), 5)) == tree_to_string(TreeNode(4, TreeNode(2, TreeNode(1, None, None), TreeNode(3, None, None)), TreeNode(7, TreeNode(5, None, None), None)))
    assert tree_to_string(insertIntoBST(TreeNode(40, TreeNode(20, TreeNode(10, None, None), TreeNode(30, None, None)), TreeNode(60, TreeNode(50, None, None), TreeNode(70, None, None))), 25)) == tree_to_string(TreeNode(40, TreeNode(20, TreeNode(10, None, None), TreeNode(30, TreeNode(25, None, None), None)), TreeNode(60, TreeNode(50, None, None), TreeNode(70, None, None))))
    assert tree_to_string(insertIntoBST(TreeNode(4, TreeNode(2, TreeNode(1, None, None), TreeNode(3, None, None)), TreeNode(7, None, None)), 5)) == tree_to_string(TreeNode(4, TreeNode(2, TreeNode(1, None, None), TreeNode(3, None, None)), TreeNode(7, TreeNode(5, None, None), None)))

if __name__ == "__main__":
    main()
