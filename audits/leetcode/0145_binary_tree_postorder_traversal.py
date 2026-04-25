
# LeetCode 145: Binary Tree Postorder Traversal
# Python version
from helpers.tree_node import TreeNode, tree_to_string
def postorderTraversal(root: TreeNode | None) -> list[int]:
    stack = [root]
    visit = [False]
    res = []

    while stack:
        cur, v = stack.pop(), visit.pop()
        if cur:
            if v:
                res.append(cur.val)
            else:
                stack.append(cur)
                visit.append(True)
                stack.append(cur.right)
                visit.append(False)
                stack.append(cur.left)
                visit.append(False)
    return res

def main():
    assert postorderTraversal(TreeNode(1, None, TreeNode(2, TreeNode(3, None, None), None))) == [3, 2, 1]
    assert postorderTraversal(TreeNode(1, TreeNode(2, TreeNode(4, None, None), TreeNode(5, TreeNode(6, None, None), TreeNode(7, None, None))), TreeNode(3, None, TreeNode(8, TreeNode(9, None, None), None)))) == [4, 6, 7, 5, 2, 9, 8, 3, 1]
    assert postorderTraversal(None) == []

if __name__ == "__main__":
    main()
