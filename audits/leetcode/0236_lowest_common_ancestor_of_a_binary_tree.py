
# LeetCode 236: Lowest Common Ancestor Of A Binary Tree
# Python version
from helpers.tree_node import TreeNode, tree_to_string
def lowestCommonAncestor(root: 'TreeNode', p: 'TreeNode', q: 'TreeNode') -> 'TreeNode':
    if not root:
        return
    if root == p or root == q:
        return root

    left = lowestCommonAncestor(root.left, p, q)
    right = lowestCommonAncestor(root.right, p, q)

    if left and right:
        return root

    if left:
        return left
    if right:
        return right

    return None

def main():
    root = TreeNode(
        3,
        TreeNode(5, TreeNode(6, None, None), TreeNode(2, TreeNode(7, None, None), TreeNode(4, None, None))),
        TreeNode(1, TreeNode(0, None, None), TreeNode(8, None, None)),
    )
    assert tree_to_string(lowestCommonAncestor(root, root.left, root.right)) == tree_to_string(root)
    assert tree_to_string(lowestCommonAncestor(root, root.left, root.left.right.right)) == tree_to_string(root.left)
    root2 = TreeNode(1, TreeNode(2, None, None), None)
    assert tree_to_string(lowestCommonAncestor(root2, root2, root2.left)) == tree_to_string(root2)

if __name__ == "__main__":
    main()
