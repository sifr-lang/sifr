from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 14: Longest Common Prefix
# Python version with test cases

def longestCommonPrefix(strs: list[str]) -> str:
    if len(strs) == 0:
        return ""
    prefix: str = strs[0]
    for i in range(1, len(strs)):
        while not strs[i].startswith(prefix):
            prefix = prefix[:len(prefix) - 1]
            if len(prefix) == 0:
                return ""
    return prefix

def main():
    assert longestCommonPrefix(["flower", "flow", "flight"]) == 'fl'
    assert longestCommonPrefix(["dog", "racecar", "car"]) == ''
    assert longestCommonPrefix(["interspecies", "interstellar", "interstate"]) == 'inters'
    assert longestCommonPrefix(["a"]) == 'a'

if __name__ == "__main__":
    main()
