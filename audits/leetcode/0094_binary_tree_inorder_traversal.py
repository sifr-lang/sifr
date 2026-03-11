# LeetCode 94: Binary Tree Inorder Traversal
# Python version

def inorderTraversal(root: Optional[TreeNode]) -> List[int]:
    # Iterative
    res, stack = [], []
    cur = root

    while cur or stack:
        while cur:
            stack.append(cur)
            cur = cur.left
        cur = stack.pop()
        res.append(cur.val)
        cur = cur.right
    return res

    # Recursive
    res = []

    def helper(root):
        if not root:
            return
        helper(root.left)
        res.append(root.val)
        helper(root.right)

    helper(root)
    return res



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
