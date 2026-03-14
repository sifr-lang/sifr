from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 3: Longest Substring Without Repeating Characters
# Python version

def lengthOfLongestSubstring(s: str) -> int:
    charSet = set()
    l = 0
    res = 0

    for r in range(len(s)):
        while s[r] in charSet:
            charSet.remove(s[l])
            l += 1
        charSet.add(s[r])
        res = max(res, r - l + 1)
    return res



def main():
    assert lengthOfLongestSubstring("abcabcbb") == 3
    assert lengthOfLongestSubstring("bbbbb") == 1
    assert lengthOfLongestSubstring("pwwkew") == 3

if __name__ == "__main__":
    main()
