
# LeetCode 543: Diameter Of Binary Tree
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


def tree_to_string(node: TreeNode | None) -> str:
    if node is None:
        return "None"
    return f"{node.val}({tree_to_string(node.left)},{tree_to_string(node.right)})"


class Node:
    def __init__(
        self,
        val: int = 0,
        next: 'Node | None' = None,
        random: 'Node | None' = None,
        left: 'Node | None' = None,
        right: 'Node | None' = None,
        neighbors: list['Node'] | None = None,
        key: int = -1,
    ):
        self.val = val
        self.next = next
        self.random = random
        self.left = left
        self.right = right
        self.neighbors = [] if neighbors is None else neighbors
        self.key = key

def diameterOfBinaryTree(root: TreeNode | None) -> int:
    res = 0

    def dfs(root):
        nonlocal res

        if not root:
            return 0
        left = dfs(root.left)
        right = dfs(root.right)
        res = max(res, left + right)

        return 1 + max(left, right)

    dfs(root)
    return res



def main():
    assert diameterOfBinaryTree(TreeNode(1, TreeNode(2, TreeNode(4, None, None), TreeNode(5, None, None)), TreeNode(3, None, None))) == 3
    assert diameterOfBinaryTree(TreeNode(1, TreeNode(2, None, None), None)) == 1

if __name__ == "__main__":
    main()