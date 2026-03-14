
# LeetCode 103: Binary Tree Zigzag Level Order Traversal
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