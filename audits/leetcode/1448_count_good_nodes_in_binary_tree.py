# LeetCode 1448: Count Good Nodes In Binary Tree
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

def goodNodes(root: TreeNode) -> int:
    def dfs(node, maxVal):
        if not node:
            return 0

        res = 1 if node.val >= maxVal else 0
        maxVal = max(maxVal, node.val)
        res += dfs(node.left, maxVal)
        res += dfs(node.right, maxVal)
        return res

    return dfs(root, root.val)



def main():
    assert goodNodes(TreeNode(3, TreeNode(1, TreeNode(3, None, None), None), TreeNode(4, TreeNode(1, None, None), TreeNode(5, None, None)))) == 4
    assert goodNodes(TreeNode(3, TreeNode(3, TreeNode(4, None, None), TreeNode(2, None, None)), None)) == 3
    assert goodNodes(TreeNode(1, None, None)) == 1

if __name__ == "__main__":
    main()