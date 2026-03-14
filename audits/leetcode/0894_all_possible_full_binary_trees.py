# LeetCode 894: All Possible Full Binary Trees
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

def allPossibleFBT(n: int) -> List[Optional[TreeNode]]:
    dp = { 0 : [], 1 : [ TreeNode() ] }

    def backtrack(n):
        if n in dp:
            return dp[n]
        
        res = []
        for l in range(n):
            r = n - 1 - l
            leftTrees, rightTrees = backtrack(l), backtrack(r)

            for t1 in leftTrees:
                for t2 in rightTrees:
                    res.append(TreeNode(0, t1, t2))
        dp[n] = res
        return res
    
    return backtrack(n)



def main():
    assert allPossibleFBT(7) == [<TreeNode object at 0x1064b7a80>, <TreeNode object at 0x106631b50>, <TreeNode object at 0x1064c1370>, <TreeNode object at 0x1064c1590>, <TreeNode object at 0x106467e50>]
    assert allPossibleFBT(3) == [<TreeNode object at 0x1064bdf40>]

if __name__ == "__main__":
    main()