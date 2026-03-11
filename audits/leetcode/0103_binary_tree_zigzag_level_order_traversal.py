# LeetCode 103: Binary Tree Zigzag Level Order Traversal
# Python version

def zigzagLevelOrder(root: Optional[TreeNode]) -> List[List[int]]:
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
    print("no test cases")

if __name__ == "__main__":
    main()
