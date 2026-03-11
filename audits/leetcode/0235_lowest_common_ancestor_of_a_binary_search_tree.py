# LeetCode 235: Lowest Common Ancestor Of A Binary Search Tree
# Python version

def lowestCommonAncestor(
    self, root: "TreeNode", p: "TreeNode", q: "TreeNode"
) -> "TreeNode":
    while True:
        if root.val < p.val and root.val < q.val:
            root = root.right
        elif root.val > p.val and root.val > q.val:
            root = root.left
        else:
            return root



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
