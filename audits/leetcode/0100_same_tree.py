
# LeetCode 100: Same Tree
# Python version
from helpers.tree_node import TreeNode, tree_to_string
def isSameTree(p: TreeNode | None, q: TreeNode | None) -> bool:
    if not p and not q:
        return True
    if p and q and p.val == q.val:
        return isSameTree(p.left, q.left) and isSameTree(p.right, q.right)
    else:
        return False

def main():
    assert isSameTree(TreeNode(1, TreeNode(2, None, None), TreeNode(3, None, None)), TreeNode(1, TreeNode(2, None, None), TreeNode(3, None, None))) == True
    assert isSameTree(TreeNode(1, TreeNode(2, None, None), None), TreeNode(1, None, TreeNode(2, None, None))) == False
    assert isSameTree(TreeNode(1, TreeNode(2, None, None), TreeNode(1, None, None)), TreeNode(1, TreeNode(1, None, None), TreeNode(2, None, None))) == False

if __name__ == "__main__":
    main()
