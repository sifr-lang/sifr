
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
    assert lowestCommonAncestor(TreeNode(3, TreeNode(5, TreeNode(6, None, None), TreeNode(2, TreeNode(7, None, None), TreeNode(4, None, None))), TreeNode(1, TreeNode(0, None, None), TreeNode(8, None, None))), 5, 1) == None
    assert lowestCommonAncestor(TreeNode(3, TreeNode(5, TreeNode(6, None, None), TreeNode(2, TreeNode(7, None, None), TreeNode(4, None, None))), TreeNode(1, TreeNode(0, None, None), TreeNode(8, None, None))), 5, 4) == None
    assert lowestCommonAncestor(TreeNode(1, TreeNode(2, None, None), None), 1, 2) == None

if __name__ == "__main__":
    main()
