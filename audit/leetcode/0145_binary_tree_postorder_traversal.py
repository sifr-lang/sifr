# LeetCode 145: Binary Tree Postorder Traversal
# Python version

def postorderTraversal(root: Optional[TreeNode]) -> List[int]:
    stack = [root]
    visit = [False]
    res = []

    while stack:
        cur, v = stack.pop(), visit.pop()
        if cur:
            if v:
                res.append(cur.val)
            else:
                stack.append(cur)
                visit.append(True)
                stack.append(cur.right)
                visit.append(False)
                stack.append(cur.left)
                visit.append(False)
    return res



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
