# LeetCode 701: Insert Into A Binary Search Tree
# Python version

def insertIntoBST(root: Optional[TreeNode], val: int) -> Optional[TreeNode]:
    if not root:
        return TreeNode(val)
    if val > root.val:
        root.right = insertIntoBST(root.right, val)
    else:
        root.left = insertIntoBST(root.left, val)
    return root



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
