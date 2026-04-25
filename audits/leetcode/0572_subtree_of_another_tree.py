
# LeetCode 572: Subtree Of Another Tree
# Python version
from helpers.tree_node import TreeNode, tree_to_string
def isSubtree(root: TreeNode | None, subRoot: TreeNode | None) -> bool:
    if not subRoot:
        return True
    if not root:
        return False

    if isSameTree(root, subRoot):
        return True
    return isSubtree(root.left, subRoot) or isSubtree(root.right, subRoot)

def isSameTree(p: TreeNode | None, q: TreeNode | None) -> bool:
    if not p and not q:
        return True
    if p and q and p.val == q.val:
        return isSameTree(p.left, q.left) and isSameTree(p.right, q.right)
    else:
        return False

def main():
    assert isSubtree(TreeNode(3, TreeNode(4, TreeNode(1, None, None), TreeNode(2, None, None)), TreeNode(5, None, None)), TreeNode(4, TreeNode(1, None, None), TreeNode(2, None, None))) == True
    assert isSubtree(TreeNode(3, TreeNode(4, TreeNode(1, None, None), TreeNode(2, TreeNode(0, None, None), None)), TreeNode(5, None, None)), TreeNode(4, TreeNode(1, None, None), TreeNode(2, None, None))) == False

if __name__ == "__main__":
    main()
