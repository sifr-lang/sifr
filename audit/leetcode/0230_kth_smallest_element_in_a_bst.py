# LeetCode 230: Kth Smallest Element In A Bst
# Python version

def kthSmallest(root: TreeNode, k: int) -> int:
    stack = []
    curr = root

    while stack or curr:
        while curr:
            stack.append(curr)
            curr = curr.left
        curr = stack.pop()
        k -= 1
        if k == 0:
            return curr.val
        curr = curr.right



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
