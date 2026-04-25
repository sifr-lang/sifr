
# LeetCode 226: Invert Binary Tree
# Python version
from helpers.tree_node import TreeNode, tree_to_string
def invertTree(root: TreeNode | None) -> TreeNode | None:
    if not root:
        return None
    
    # swap the children
    root.left, root.right = root.right, root.left
    
    # make 2 recursive calls
    invertTree(root.left)
    invertTree(root.right)
    return root

def main():
    assert tree_to_string(invertTree(TreeNode(4, TreeNode(2, TreeNode(1, None, None), TreeNode(3, None, None)), TreeNode(7, TreeNode(6, None, None), TreeNode(9, None, None))))) == tree_to_string(TreeNode(4, TreeNode(7, TreeNode(9, None, None), TreeNode(6, None, None)), TreeNode(2, TreeNode(3, None, None), TreeNode(1, None, None))))
    assert tree_to_string(invertTree(TreeNode(2, TreeNode(1, None, None), TreeNode(3, None, None)))) == tree_to_string(TreeNode(2, TreeNode(3, None, None), TreeNode(1, None, None)))
    assert invertTree(None) == None

if __name__ == "__main__":
    main()
