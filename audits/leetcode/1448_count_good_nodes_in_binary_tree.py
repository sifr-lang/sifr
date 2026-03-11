# LeetCode 1448: Count Good Nodes In Binary Tree
# Python version

def goodNodes(root: TreeNode) -> int:
    def dfs(node, maxVal):
        if not node:
            return 0

        res = 1 if node.val >= maxVal else 0
        maxVal = max(maxVal, node.val)
        res += dfs(node.left, maxVal)
        res += dfs(node.right, maxVal)
        return res

    return dfs(root, root.val)



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
