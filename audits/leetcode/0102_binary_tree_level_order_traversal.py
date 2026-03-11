# LeetCode 102: Binary Tree Level Order Traversal
# Python version

def levelOrder(root: TreeNode) -> List[List[int]]:
    res = []
    q = collections.deque()
    if root:
        q.append(root)

    while q:
        val = []

        for i in range(len(q)):
            node = q.popleft()
            val.append(node.val)
            if node.left:
                q.append(node.left)
            if node.right:
                q.append(node.right)
        res.append(val)
    return res



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
