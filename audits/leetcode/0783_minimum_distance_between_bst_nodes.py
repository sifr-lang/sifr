
# LeetCode 783: Minimum Distance Between Bst Nodes
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

def minDiffInBST(root: TreeNode | None) -> int:
    curr_smallest, prev = float("inf"), None
    
    def inorder(node):
        nonlocal curr_smallest, prev
        if node is None:
            return
        
        inorder(node.left)
        if prev is not None:
            curr_smallest = min(curr_smallest, node.val-prev.val)
        prev = node
        inorder(node.right)

    inorder(root)
    return curr_smallest



def main():
    assert minDiffInBST(TreeNode(4, TreeNode(2, TreeNode(1, None, None), TreeNode(3, None, None)), TreeNode(6, None, None))) == 1
    assert minDiffInBST(TreeNode(1, TreeNode(0, None, None), TreeNode(48, TreeNode(12, None, None), TreeNode(49, None, None)))) == 1

if __name__ == "__main__":
    main()