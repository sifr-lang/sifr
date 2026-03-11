# LeetCode 124: Binary Tree Maximum Path Sum
# Python version

def maxPathSum(root: TreeNode) -> int:
    res = [root.val]

    # return max path sum without split
    def dfs(root):
        if not root:
            return 0

        leftMax = dfs(root.left)
        rightMax = dfs(root.right)
        leftMax = max(leftMax, 0)
        rightMax = max(rightMax, 0)

        # compute max path sum WITH split
        res[0] = max(res[0], root.val + leftMax + rightMax)
        return root.val + max(leftMax, rightMax)

    dfs(root)
    return res[0]



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
