
# LeetCode 98: Validate Binary Search Tree
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