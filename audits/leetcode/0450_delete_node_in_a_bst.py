
# LeetCode 450: Delete Node In A Bst
# Python version
from helpers.tree_node import TreeNode, tree_to_string
def deleteNode(root: TreeNode | None, key: int) -> TreeNode | None:
    if not root:
        return root
    
    if key > root.val:
        root.right = deleteNode(root.right, key)
    elif key < root.val:
        root.left = deleteNode(root.left, key)
    else:
        if not root.left:
            return root.right
        elif not root.right:
            return root.left
        
        # Find the min from right subtree
        cur = root.right
        while cur.left:
            cur = cur.left 
        root.val = cur.val
        root.right = deleteNode(root.right, root.val)
    return root

def main():
    assert tree_to_string(deleteNode(TreeNode(5, TreeNode(3, TreeNode(2, None, None), TreeNode(4, None, None)), TreeNode(6, None, TreeNode(7, None, None))), 3)) == tree_to_string(TreeNode(5, TreeNode(4, TreeNode(2, None, None), None), TreeNode(6, None, TreeNode(7, None, None))))
    assert tree_to_string(deleteNode(TreeNode(5, TreeNode(3, TreeNode(2, None, None), TreeNode(4, None, None)), TreeNode(6, None, TreeNode(7, None, None))), 0)) == tree_to_string(TreeNode(5, TreeNode(3, TreeNode(2, None, None), TreeNode(4, None, None)), TreeNode(6, None, TreeNode(7, None, None))))
    assert deleteNode(None, 0) == None

if __name__ == "__main__":
    main()
