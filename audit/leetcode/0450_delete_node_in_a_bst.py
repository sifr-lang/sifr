# LeetCode 450: Delete Node In A Bst
# Python version

def deleteNode(root: Optional[TreeNode], key: int) -> Optional[TreeNode]:
    if not root:
        return root
    
    if key > root.val:
        root.right = deleteNode(root.right, key)
    elif key < root.val:
        root.left = deleteNode(root.left, key)
    else:
        if not root.left:
            return root.right
        elif not root.right:
            return root.left
        
        # Find the min from right subtree
        cur = root.right
        while cur.left:
            cur = cur.left 
        root.val = cur.val
        root.right = deleteNode(root.right, root.val)
    return root



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
