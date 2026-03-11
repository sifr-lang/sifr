# LeetCode 669: Trim A Binary Search Tree
# Python version

def trimBST(root: Optional[TreeNode], low: int, high: int) -> Optional[TreeNode]:
    if not root:
        return None

    if root.val > high:
        return trimBST(root.left, low, high)

    if root.val < low:
        return trimBST(root.right, low, high)

    else:
        root.left = trimBST(root.left, low, high)
        root.right = trimBST(root.right, low, high)
        return root


def main():
    print("no test cases")

if __name__ == "__main__":
    main()
