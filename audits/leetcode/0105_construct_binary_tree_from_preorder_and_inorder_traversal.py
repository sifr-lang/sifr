# LeetCode 105: Construct Binary Tree From Preorder And Inorder Traversal
# Python version

def buildTree(preorder: List[int], inorder: List[int]) -> Optional[TreeNode]:
    if not preorder or not inorder:
        return None

    root = TreeNode(preorder[0])
    mid = inorder.index(preorder[0])
    root.left = buildTree(preorder[1 : mid + 1], inorder[:mid])
    root.right = buildTree(preorder[mid + 1 :], inorder[mid + 1 :])
    return root



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
