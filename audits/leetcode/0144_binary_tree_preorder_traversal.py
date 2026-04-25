
# LeetCode 144: Binary Tree Preorder Traversal
# Python version
from helpers.tree_node import TreeNode, tree_to_string
def preorderTraversal(root: TreeNode | None) -> list[int]:
    cur, stack = root, []
    res = []
    while cur or stack:
        if cur:
            res.append(cur.val)
            stack.append(cur.right)
            cur = cur.left
        else:
            cur = stack.pop()
    return res

def main():
    assert preorderTraversal(TreeNode(1, None, TreeNode(2, TreeNode(3, None, None), None))) == [1, 2, 3]
    assert preorderTraversal(TreeNode(1, TreeNode(2, TreeNode(4, None, None), TreeNode(5, TreeNode(6, None, None), TreeNode(7, None, None))), TreeNode(3, None, TreeNode(8, TreeNode(9, None, None), None)))) == [1, 2, 4, 5, 6, 7, 3, 8, 9]
    assert preorderTraversal(None) == []

if __name__ == "__main__":
    main()
