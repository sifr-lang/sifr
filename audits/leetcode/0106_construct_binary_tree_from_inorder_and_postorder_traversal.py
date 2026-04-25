
# LeetCode 106: Construct Binary Tree From Inorder And Postorder Traversal
# Python version
from helpers.tree_node import TreeNode, tree_to_string
def buildTree(inorder: list[int], postorder: list[int]) -> TreeNode | None:
    def buildTreeHelper(left, right):
        if left > right:
            return None

        rootVal = postorder.pop()
        rootNode = TreeNode(rootVal)

        idx = inorderIndexMap[rootVal]
        rootNode.right = buildTreeHelper(idx + 1, right)
        rootNode.left = buildTreeHelper(left, idx - 1)
        return rootNode

    inorderIndexMap = {}
    for (i, val) in enumerate(inorder):
        inorderIndexMap[val] = i

    return buildTreeHelper(0, len(postorder) - 1)

def main():
    assert tree_to_string(buildTree([9, 3, 15, 20, 7], [9, 15, 7, 20, 3])) == "3(9(None,None),20(15(None,None),7(None,None)))"
    assert tree_to_string(buildTree([-1], [-1])) == "-1(None,None)"

if __name__ == "__main__":
    main()
