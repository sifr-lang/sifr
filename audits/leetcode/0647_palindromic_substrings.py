from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 647: Palindromic Substrings
# Python version

def countSubstrings(s: str) -> int:
    res = 0

    for i in range(len(s)):
        res += countPali(s, i, i)
        res += countPali(s, i, i + 1)
    return res


def countPali(s, l, r):
    res = 0
    while l >= 0 and r < len(s) and s[l] == s[r]:
        res += 1
        l -= 1
        r += 1
    return res



def main():
    assert countSubstrings("abc") == 3
    assert countSubstrings("aaa") == 6

if __name__ == "__main__":
    main()
