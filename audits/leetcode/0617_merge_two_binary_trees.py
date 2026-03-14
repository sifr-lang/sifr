# LeetCode 617: Merge Two Binary Trees
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

def mergeTrees(t1: TreeNode, t2: TreeNode) -> TreeNode:
    if not t1 and not t2:
        return None

    v1 = t1.val if t1 else 0
    v2 = t2.val if t2 else 0
    root = TreeNode(v1 + v2)

    root.left = mergeTrees(t1.left if t1 else None, t2.left if t2 else None)
    root.right = mergeTrees(t1.right if t1 else None, t2.right if t2 else None)
    return root



def main():
    assert mergeTrees(TreeNode(1, TreeNode(3, TreeNode(5, None, None), None), TreeNode(2, None, None)), TreeNode(2, TreeNode(1, None, TreeNode(4, None, None)), TreeNode(3, None, TreeNode(7, None, None)))) == <TreeNode object at 0x106724440>
    assert mergeTrees(TreeNode(1, None, None), TreeNode(1, TreeNode(2, None, None), None)) == <TreeNode object at 0x1064c0160>

if __name__ == "__main__":
    main()