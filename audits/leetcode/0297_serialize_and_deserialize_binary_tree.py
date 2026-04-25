
# LeetCode 297: Serialize And Deserialize Binary Tree
# Python version
from helpers.tree_node import TreeNode, tree_to_string
class Codec:
    def serialize(self, root):
        res = []
        def dfs(node):
            if not node:
                res.append("N")
                return
            res.append(str(node.val))
            dfs(node.left)
            dfs(node.right)
        dfs(root)
        return ",".join(res)
    def deserialize(self, data):
        vals = data.split(",")
        def dfs():
            val = vals.pop(0)
            if val == "N":
                return None
            node = TreeNode(val=int(val))
            node.left = dfs()
            node.right = dfs()
            return node
        return dfs()

def main():
    root = TreeNode(1, TreeNode(2, None, None), TreeNode(3, TreeNode(4, None, None), TreeNode(5, None, None)))
    codec = Codec()
    assert tree_to_string(codec.deserialize(codec.serialize(root))) == tree_to_string(root)
    root = None
    codec = Codec()
    assert tree_to_string(codec.deserialize(codec.serialize(root))) == tree_to_string(root)

if __name__ == "__main__":
    main()
