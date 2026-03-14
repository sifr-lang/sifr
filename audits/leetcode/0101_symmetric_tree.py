from collections import deque

# LeetCode 101: Symmetric Tree
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

def isSymmetric(root: TreeNode | None) -> bool:
    if not root.left and not root.right:
        return True
    queueLeft = deque()
    queueRight = deque()

    queueLeft.appendleft(root.left)
    queueRight.appendleft(root.right)

    while queueLeft and queueRight:
        nodeLeft, nodeRight = queueLeft.pop(), queueRight.pop()
        if not nodeLeft and not nodeRight:
            continue
        # both node must exist
        # if exists thet must have the same value
        if not nodeLeft or not nodeRight or nodeLeft.val != nodeRight.val:
            return False

        queueLeft.appendleft(nodeLeft.left)
        queueLeft.appendleft(nodeLeft.right)

        queueRight.appendleft(nodeRight.right)
        queueRight.appendleft(nodeRight.left)
    return not (queueLeft or queueRight)



def main():
    assert isSymmetric(TreeNode(1, TreeNode(2, TreeNode(3, None, None), TreeNode(4, None, None)), TreeNode(2, TreeNode(4, None, None), TreeNode(3, None, None)))) == True
    assert isSymmetric(TreeNode(1, TreeNode(2, None, TreeNode(3, None, None)), TreeNode(2, None, TreeNode(3, None, None)))) == False

if __name__ == "__main__":
    main()