# LeetCode 662: Maximum Width Of Binary Tree
# Python version

def widthOfBinaryTree(root: Optional[TreeNode]) -> int:
    if root is None:
        return 0

    q = [(root, 0)]
    width = 0
    while q:
        leftIndex = q[0][1]
        rightIndex = q[-1][1]
        width = max(width, rightIndex - leftIndex + 1)

        for _ in range(len(q)):
            node, index = q.pop(0)
            if node.left:
                q.append((node.left, index * 2))
            if node.right:
                q.append((node.right, index * 2 + 1))
    return width



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
