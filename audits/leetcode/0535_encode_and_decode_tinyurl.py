from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

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
    codec = Codec()
    url = "https://leetcode.com/problems/design-tinyurl"
    short_url = codec.encode(url)
    assert short_url == "http://tinyurl.com/1"
    assert codec.decode(short_url) == url
    assert codec.encode(url) == short_url

if __name__ == "__main__":
    main()
