
# LeetCode 103: Binary Tree Zigzag Level Order Traversal
# Python version
from helpers.tree_node import TreeNode, tree_to_string
def zigzagLevelOrder(root: TreeNode | None) -> list[list[int]]:
    if root is None:
        return
    result, zigzagDirection = [], 1
    q = [root]
    while q:
        level, queueLength = [], len(q)
        for i in range(queueLength):
            node = q.pop(0)
            level.append(node.val)
            if node.left:
                q.append(node.left)
            if node.right:
                q.append(node.right)
        result.append(level[::zigzagDirection])
        zigzagDirection *= -1
    return result

def main():
    assert zigzagLevelOrder(TreeNode(3, TreeNode(9, None, None), TreeNode(20, TreeNode(15, None, None), TreeNode(7, None, None)))) == [[3], [20, 9], [15, 7]]
    assert zigzagLevelOrder(TreeNode(1, None, None)) == [[1]]
    assert zigzagLevelOrder(None) == None

if __name__ == "__main__":
    main()
