
# LeetCode 235: Lowest Common Ancestor Of A Binary Search Tree
# Python version
from helpers.tree_node import TreeNode, tree_to_string
def lowestCommonAncestor(
    self, root: "TreeNode", p: "TreeNode", q: "TreeNode"
) -> "TreeNode":
    while True:
        if root.val < p.val and root.val < q.val:
            root = root.right
        elif root.val > p.val and root.val > q.val:
            root = root.left
        else:
            return root

def main():
    root = TreeNode(
        6,
        TreeNode(2, TreeNode(0), TreeNode(4, TreeNode(3), TreeNode(5))),
        TreeNode(8, TreeNode(7), TreeNode(9)),
    )
    assert tree_to_string(
        lowestCommonAncestor(None, root, root.left, root.right)
    ) == tree_to_string(root)
    assert tree_to_string(
        lowestCommonAncestor(None, root, root.left, root.left.right)
    ) == tree_to_string(root.left)
    assert tree_to_string(
        lowestCommonAncestor(None, TreeNode(2, TreeNode(1), None), TreeNode(2), TreeNode(1))
    ) == tree_to_string(TreeNode(2, TreeNode(1), None))

if __name__ == "__main__":
    main()
