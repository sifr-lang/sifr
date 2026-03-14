
# LeetCode 701: Insert Into A Binary Search Tree
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

def insertIntoBST(root: TreeNode | None, val: int) -> TreeNode | None:
    if not root:
        return TreeNode(val)
    if val > root.val:
        root.right = insertIntoBST(root.right, val)
    else:
        root.left = insertIntoBST(root.left, val)
    return root



def main():
    assert tree_to_string(insertIntoBST(TreeNode(4, TreeNode(2, TreeNode(1, None, None), TreeNode(3, None, None)), TreeNode(7, None, None)), 5)) == tree_to_string(TreeNode(4, TreeNode(2, TreeNode(1, None, None), TreeNode(3, None, None)), TreeNode(7, TreeNode(5, None, None), None)))
    assert tree_to_string(insertIntoBST(TreeNode(40, TreeNode(20, TreeNode(10, None, None), TreeNode(30, None, None)), TreeNode(60, TreeNode(50, None, None), TreeNode(70, None, None))), 25)) == tree_to_string(TreeNode(40, TreeNode(20, TreeNode(10, None, None), TreeNode(30, TreeNode(25, None, None), None)), TreeNode(60, TreeNode(50, None, None), TreeNode(70, None, None))))
    assert tree_to_string(insertIntoBST(TreeNode(4, TreeNode(2, TreeNode(1, None, None), TreeNode(3, None, None)), TreeNode(7, None, None)), 5)) == tree_to_string(TreeNode(4, TreeNode(2, TreeNode(1, None, None), TreeNode(3, None, None)), TreeNode(7, TreeNode(5, None, None), None)))

if __name__ == "__main__":
    main()