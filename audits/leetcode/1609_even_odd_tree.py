from helpers.tree_node import TreeNode, tree_to_string

from collections import deque

# LeetCode 1609: Even Odd Tree
# Python version
def isEvenOddTree(root: TreeNode | None) -> bool:
    even = True
    q = deque([root])

    while q:
        prev = float("-inf") if even else float("inf")
        for _ in range(len(q)):
            node = q.popleft()

            if even and (node.val % 2 == 0 or node.val <= prev):
                return False
            elif not even and (node.val % 2 == 1 or node.val >= prev):
                return False
            
            if node.left:
                q.append(node.left)
            if node.right:
                q.append(node.right)
            prev = node.val
        even = not even
    return True

def main():
    assert isEvenOddTree(TreeNode(1, TreeNode(10, TreeNode(3, TreeNode(12, None, None), TreeNode(8, None, None)), None), TreeNode(4, TreeNode(7, TreeNode(6, None, None), None), TreeNode(9, None, TreeNode(2, None, None))))) == True
    assert isEvenOddTree(TreeNode(5, TreeNode(4, TreeNode(3, None, None), TreeNode(3, None, None)), TreeNode(2, TreeNode(7, None, None), None))) == False
    assert isEvenOddTree(TreeNode(5, TreeNode(9, TreeNode(3, None, None), TreeNode(5, None, None)), TreeNode(1, TreeNode(7, None, None), None))) == False

if __name__ == "__main__":
    main()
