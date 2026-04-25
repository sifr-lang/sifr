
# LeetCode 535: Encode And Decode Tinyurl
# Python version
from helpers.tree_node import TreeNode, tree_to_string
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
    codec = Codec()
    url = "https://leetcode.com/problems/design-tinyurl"
    short_url = codec.encode(url)
    assert short_url == "http://tinyurl.com/1"
    assert codec.decode(short_url) == url
    assert codec.encode(url) == short_url

if __name__ == "__main__":
    main()
