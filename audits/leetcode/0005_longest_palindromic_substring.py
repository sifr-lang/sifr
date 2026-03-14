from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 5: Longest Palindromic Substring
# Python version

def longestPalindrome(s: str) -> str:
    res = ""
    lenres = 0
    for i in range(len(s)):
        s1 = helper(s, i, i)
        s2 = helper(s, i, i + 1)
        if len(s1) > lenres:
            res = s1
            lenres = len(s1)
        if len(s2) > lenres:
            res = s2
            lenres = len(s2)
    return res
    

def helper(s, left, right):
        while left >= 0 and right < len(s) and s[left] == s[right]:
            left -= 1
            right += 1
        return s[left + 1:right]





def main():
    assert longestPalindrome("babad",) in {"bab", "aba"}
    assert longestPalindrome("cbbd",) == 'bb'

if __name__ == "__main__":
    main()
