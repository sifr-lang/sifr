# LeetCode 236: Lowest Common Ancestor Of A Binary Tree
# Python version

def lowestCommonAncestor(root: 'TreeNode', p: 'TreeNode', q: 'TreeNode') -> 'TreeNode':
    if not root:
        return
    if root == p or root == q:
        return root

    left = lowestCommonAncestor(root.left, p, q)
    right = lowestCommonAncestor(root.right, p, q)

    if left and right:
        return root

    if left:
        return left
    if right:
        return right

    return None



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
