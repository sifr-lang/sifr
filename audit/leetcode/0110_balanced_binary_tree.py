# LeetCode 110: Balanced Binary Tree
# Python version

def isBalanced(root: Optional[TreeNode]) -> bool:
    def dfs(root):
        if not root:
            return [True, 0]

        left, right = dfs(root.left), dfs(root.right)
        balanced = left[0] and right[0] and abs(left[1] - right[1]) <= 1
        return [balanced, 1 + max(left[1], right[1])]

    return dfs(root)[0]



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
