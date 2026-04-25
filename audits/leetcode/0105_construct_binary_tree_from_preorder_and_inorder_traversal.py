
# LeetCode 105: Construct Binary Tree From Preorder And Inorder Traversal
# Python version
from helpers.tree_node import TreeNode, tree_to_string
def buildTree(preorder: list[int], inorder: list[int]) -> TreeNode | None:
    if not preorder or not inorder:
        return None

    root = TreeNode(preorder[0])
    mid = inorder.index(preorder[0])
    root.left = buildTree(preorder[1 : mid + 1], inorder[:mid])
    root.right = buildTree(preorder[mid + 1 :], inorder[mid + 1 :])
    return root

def main():
    assert tree_to_string(buildTree([3, 9, 20, 15, 7], [9, 3, 15, 20, 7])) == "3(9(None,None),20(15(None,None),7(None,None)))"
    assert tree_to_string(buildTree([-1], [-1])) == "-1(None,None)"

if __name__ == "__main__":
    main()
