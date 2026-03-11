# LeetCode 104: Maximum Depth Of Binary Tree
# Python version

def maxDepth(root: TreeNode) -> int:
    if not root:
        return 0

    return 1 + max(maxDepth(root.left), maxDepth(root.right))


# ITERATIVE DFS

def maxDepth(root: TreeNode) -> int:
    stack = [[root, 1]]
    res = 0

    while stack:
        node, depth = stack.pop()

        if node:
            res = max(res, depth)
            stack.append([node.left, depth + 1])
            stack.append([node.right, depth + 1])
    return res


# BFS

def maxDepth(root: TreeNode) -> int:
    q = deque()
    if root:
        q.append(root)

    level = 0

    while q:

        for i in range(len(q)):
            node = q.popleft()
            if node.left:
                q.append(node.left)
            if node.right:
                q.append(node.right)
        level += 1
    return level



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
