# LeetCode 226: Invert Binary Tree
# Python version

def invertTree(root: Optional[TreeNode]) -> Optional[TreeNode]:
    if not root:
        return None
    
    # swap the children
    root.left, root.right = root.right, root.left
    
    # make 2 recursive calls
    invertTree(root.left)
    invertTree(root.right)
    return root



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
