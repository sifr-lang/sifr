
# LeetCode 108: Convert Sorted Array To Binary Search Tree
# Python version

class TreeNode:
    def __init__(
        self,
        val: int = 0,
        left: 'TreeNode | None' = None,
        right: 'TreeNode | None' = None,
    ):
        self.val = val
        self.left = left
        self.right = right


def tree_to_string(node: TreeNode | None) -> str:
    if node is None:
        return "None"
    return f"{node.val}({tree_to_string(node.left)},{tree_to_string(node.right)})"


class Node:
    def __init__(
        self,
        val: int = 0,
        next: 'Node | None' = None,
        random: 'Node | None' = None,
        left: 'Node | None' = None,
        right: 'Node | None' = None,
        neighbors: list['Node'] | None = None,
        key: int = -1,
    ):
        self.val = val
        self.next = next
        self.random = random
        self.left = left
        self.right = right
        self.neighbors = [] if neighbors is None else neighbors
        self.key = key

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
