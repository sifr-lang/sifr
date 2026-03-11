# LeetCode 543: Diameter Of Binary Tree
# Python version

def diameterOfBinaryTree(root: Optional[TreeNode]) -> int:
    res = 0

    def dfs(root):
        nonlocal res

        if not root:
            return 0
        left = dfs(root.left)
        right = dfs(root.right)
        res = max(res, left + right)

        return 1 + max(left, right)

    dfs(root)
    return res



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
