
# LeetCode 230: Kth Smallest Element In A Bst
# Python version
from helpers.tree_node import TreeNode, tree_to_string
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
