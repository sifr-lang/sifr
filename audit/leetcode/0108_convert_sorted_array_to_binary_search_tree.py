# LeetCode 108: Convert Sorted Array To Binary Search Tree
# Python version

def sortedArrayToBST(nums: List[int]) -> Optional[TreeNode]:
    if not nums:
        return None
    mid = len(nums)//2
    root = TreeNode(nums[mid])
    root.left = sortedArrayToBST(nums[:mid])
    root.right = sortedArrayToBST(nums[mid+1:])
    return root



def main():
    print("no test cases")

if __name__ == "__main__":
    main()
