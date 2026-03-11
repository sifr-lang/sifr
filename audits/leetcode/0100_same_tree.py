# LeetCode 100: Same Tree
# Python version

def isSameTree(p: Optional[TreeNode], q: Optional[TreeNode]) -> bool:
    if not p and not q:
        return True
    if p and q and p.val == q.val:
        return isSameTree(p.left, q.left) and isSameTree(p.right, q.right)
    else:
        return False



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
