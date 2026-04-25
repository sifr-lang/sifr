
# LeetCode 662: Maximum Width Of Binary Tree
# Python version
from helpers.tree_node import TreeNode, tree_to_string
def widthOfBinaryTree(root: TreeNode | None) -> int:
    if root is None:
        return 0

    q = [(root, 0)]
    width = 0
    while q:
        leftIndex = q[0][1]
        rightIndex = q[-1][1]
        width = max(width, rightIndex - leftIndex + 1)

        for _ in range(len(q)):
            node, index = q.pop(0)
            if node.left:
                q.append((node.left, index * 2))
            if node.right:
                q.append((node.right, index * 2 + 1))
    return width

def main():
    assert widthOfBinaryTree(TreeNode(1, TreeNode(3, TreeNode(5, None, None), TreeNode(3, None, None)), TreeNode(2, None, TreeNode(9, None, None)))) == 4
    assert widthOfBinaryTree(TreeNode(1, TreeNode(3, TreeNode(5, TreeNode(6, None, None), None), None), TreeNode(2, None, TreeNode(9, TreeNode(7, None, None), None)))) == 7
    assert widthOfBinaryTree(TreeNode(1, TreeNode(3, TreeNode(5, None, None), None), TreeNode(2, None, None))) == 2

if __name__ == "__main__":
    main()
