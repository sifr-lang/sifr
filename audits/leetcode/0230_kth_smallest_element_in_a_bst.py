
# LeetCode 230: Kth Smallest Element In A Bst
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

def kthSmallest(root: TreeNode, k: int) -> int:
    stack = []
    curr = root

    while stack or curr:
        while curr:
            stack.append(curr)
            curr = curr.left
        curr = stack.pop()
        k -= 1
        if k == 0:
            return curr.val
        curr = curr.right



def main():
    assert kthSmallest(TreeNode(3, TreeNode(1, None, TreeNode(2, None, None)), TreeNode(4, None, None)), 1) == 1
    assert kthSmallest(TreeNode(5, TreeNode(3, TreeNode(2, TreeNode(1, None, None), None), TreeNode(4, None, None)), TreeNode(6, None, None)), 3) == 3

if __name__ == "__main__":
    main()