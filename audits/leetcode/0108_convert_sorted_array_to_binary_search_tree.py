
# LeetCode 108: Convert Sorted Array To Binary Search Tree
# Python version
from helpers.tree_node import TreeNode, tree_to_string
def sortedArrayToBST(nums: list[int]) -> TreeNode | None:
    if not nums:
        return None
    mid = len(nums)//2
    root = TreeNode(nums[mid])
    root.left = sortedArrayToBST(nums[:mid])
    root.right = sortedArrayToBST(nums[mid+1:])
    return root

def main():
    assert tree_to_string(sortedArrayToBST([-10, -3, 0, 5, 9])) == "0(-3(-10(None,None),None),9(5(None,None),None))"
    assert tree_to_string(sortedArrayToBST([1, 3])) == "3(1(None,None),None)"

if __name__ == "__main__":
    main()
