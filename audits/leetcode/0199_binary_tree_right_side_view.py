import collections

# LeetCode 199: Binary Tree Right Side View
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

def rightSideView(root: TreeNode) -> list[int]:
    res = []
    q = collections.deque([root])

    while q:
        rightSide = None
        qLen = len(q)

        for i in range(qLen):
            node = q.popleft()
            if node:
                rightSide = node
                q.append(node.left)
                q.append(node.right)
        if rightSide:
            res.append(rightSide.val)
    return res



def main():
    assert rightSideView(TreeNode(1, TreeNode(2, None, TreeNode(5, None, None)), TreeNode(3, None, TreeNode(4, None, None)))) == [1, 3, 4]
    assert rightSideView(TreeNode(1, TreeNode(2, TreeNode(4, TreeNode(5, None, None), None), None), TreeNode(3, None, None))) == [1, 3, 4, 5]
    assert rightSideView(TreeNode(1, None, TreeNode(3, None, None))) == [1, 3]

if __name__ == "__main__":
    main()