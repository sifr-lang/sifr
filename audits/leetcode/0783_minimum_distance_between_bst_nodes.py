
# LeetCode 783: Minimum Distance Between Bst Nodes
# Python version
from helpers.tree_node import TreeNode, tree_to_string
def minDiffInBST(root: TreeNode | None) -> int:
    curr_smallest, prev = float("inf"), None
    
    def inorder(node):
        nonlocal curr_smallest, prev
        if node is None:
            return
        
        inorder(node.left)
        if prev is not None:
            curr_smallest = min(curr_smallest, node.val-prev.val)
        prev = node
        inorder(node.right)

    inorder(root)
    return curr_smallest

def main():
    assert minDiffInBST(TreeNode(4, TreeNode(2, TreeNode(1, None, None), TreeNode(3, None, None)), TreeNode(6, None, None))) == 1
    assert minDiffInBST(TreeNode(1, TreeNode(0, None, None), TreeNode(48, TreeNode(12, None, None), TreeNode(49, None, None)))) == 1

if __name__ == "__main__":
    main()
