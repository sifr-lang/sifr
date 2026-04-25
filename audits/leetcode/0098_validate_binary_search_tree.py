
# LeetCode 98: Validate Binary Search Tree
# Python version
from helpers.tree_node import TreeNode, tree_to_string
def isValidBST(root: TreeNode) -> bool:
    def valid(node, left, right):
        if not node:
            return True
        if not (left < node.val < right):
            return False

        return valid(node.left, left, node.val) and valid(
            node.right, node.val, right
        )

    return valid(root, float("-inf"), float("inf"))

def main():
    assert isValidBST(TreeNode(2, TreeNode(1, None, None), TreeNode(3, None, None))) == True
    assert isValidBST(TreeNode(5, TreeNode(1, None, None), TreeNode(4, TreeNode(3, None, None), TreeNode(6, None, None)))) == False

if __name__ == "__main__":
    main()
