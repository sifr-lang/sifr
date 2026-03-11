# LeetCode 144: Binary Tree Preorder Traversal
# Python version

def preorderTraversal(root: Optional[TreeNode]) -> List[int]:
    cur, stack = root, []
    res = []
    while cur or stack:
        if cur:
            res.append(cur.val)
            stack.append(cur.right)
            cur = cur.left
        else:
            cur = stack.pop()
    return res



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
