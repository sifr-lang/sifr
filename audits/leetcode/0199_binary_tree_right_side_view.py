from helpers.tree_node import TreeNode, tree_to_string

import collections

# LeetCode 199: Binary Tree Right Side View
# Python version
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
