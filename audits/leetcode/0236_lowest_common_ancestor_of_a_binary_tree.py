# LeetCode 236: Lowest Common Ancestor Of A Binary Tree
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

def lowestCommonAncestor(root: 'TreeNode', p: 'TreeNode', q: 'TreeNode') -> 'TreeNode':
    if not root:
        return
    if root == p or root == q:
        return root

    left = lowestCommonAncestor(root.left, p, q)
    right = lowestCommonAncestor(root.right, p, q)

    if left and right:
        return root

    if left:
        return left
    if right:
        return right

    return None



def main():
    assert lowestCommonAncestor(TreeNode(3, TreeNode(5, TreeNode(6, None, None), TreeNode(2, TreeNode(7, None, None), TreeNode(4, None, None))), TreeNode(1, TreeNode(0, None, None), TreeNode(8, None, None))), 5, 1) == None
    assert lowestCommonAncestor(TreeNode(3, TreeNode(5, TreeNode(6, None, None), TreeNode(2, TreeNode(7, None, None), TreeNode(4, None, None))), TreeNode(1, TreeNode(0, None, None), TreeNode(8, None, None))), 5, 4) == None
    assert lowestCommonAncestor(TreeNode(1, TreeNode(2, None, None), None), 1, 2) == None

if __name__ == "__main__":
    main()