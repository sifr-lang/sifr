# LeetCode 535: Encode And Decode Tinyurl
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

class Codec:
    def __init__(self):
        self.encodeMap = {}
        self.decodeMap = {}
        self.base = "http://tinyurl.com/"
    def encode(self, longUrl: str) -> str:
        """Encodes a URL to a shortened URL.
        """
        if longUrl not in self.encodeMap: 
            shortUrl = self.base + str(len(self.encodeMap) + 1)
            self.encodeMap[longUrl] = shortUrl
            self.decodeMap[shortUrl] = longUrl
        return self.encodeMap[longUrl]
    def decode(self, shortUrl: str) -> str:
        """Decodes a shortened URL to its original URL.
        """
        return self.decodeMap[shortUrl]

def main():
    root = TreeNode('h', TreeNode('t', TreeNode('p', TreeNode('/', TreeNode('e', TreeNode('e', None, None), TreeNode('s', None, None)), TreeNode('.', TreeNode('i', None, None), TreeNode('g', None, None))), TreeNode('l', TreeNode('c', TreeNode('n', None, None), TreeNode('-', None, None)), TreeNode('o', TreeNode('t', None, None), TreeNode('i', None, None)))), TreeNode('s', TreeNode('e', TreeNode('m', TreeNode('n', None, None), TreeNode('y', None, None)), TreeNode('/', TreeNode('u', None, None), TreeNode('r', None, None))), TreeNode('e', TreeNode('p', TreeNode('l', None, None), None), TreeNode('r', None, None)))), TreeNode('t', TreeNode(':', TreeNode('t', TreeNode('o', None, None), TreeNode('b', None, None)), TreeNode('c', TreeNode('l', None, None), TreeNode('e', None, None))), TreeNode('/', TreeNode('o', TreeNode('m', None, None), TreeNode('s', None, None)), TreeNode('d', TreeNode('/', None, None), TreeNode('d', None, None)))))
    codec = Codec()
    assert tree_to_string(codec.deserialize(codec.serialize(root))) == tree_to_string(root)

if __name__ == "__main__":
    main()