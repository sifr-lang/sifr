# LeetCode 104: Maximum Depth Of Binary Tree
# Python version

class TreeNode:
    def __init__(
        self,
        val: int = 0,
        left: 'TreeNode | None' = None,
        right: 'TreeNode | None' = None,
    ):
        self.val = val
        self.left = left
        self.right = right


def maxDepth(root: TreeNode | None) -> int:
    if root is None:
        return 0

    return 1 + max(maxDepth(root.left), maxDepth(root.right))


def main():
    assert maxDepth(TreeNode(3, TreeNode(9, None, None), TreeNode(20, TreeNode(15, None, None), TreeNode(7, None, None)))) == 3
    assert maxDepth(TreeNode(1, None, TreeNode(2, None, None))) == 2

if __name__ == "__main__":
    main()
