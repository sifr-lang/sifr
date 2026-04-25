
# LeetCode 1448: Count Good Nodes In Binary Tree
# Python version
from helpers.tree_node import TreeNode, tree_to_string
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
