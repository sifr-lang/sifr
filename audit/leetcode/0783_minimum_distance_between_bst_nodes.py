# LeetCode 783: Minimum Distance Between Bst Nodes
# Python version

def minDiffInBST(root: Optional[TreeNode]) -> int:
    curr_smallest, prev = float("inf"), None
    
    def inorder(node):
        nonlocal curr_smallest, prev
        if node is None:
            return
        
        inorder(node.left)
        if prev is not None:
            curr_smallest = min(curr_smallest, node.val-prev.val)
        prev = node
        inorder(node.right)

    inorder(root)
    return curr_smallest



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
