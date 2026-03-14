from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 278: First Bad Version
# Python version

BAD_VERSION = 4

def isBadVersion(version: int) -> bool:
    return version >= BAD_VERSION

def firstBadVersion(n: int) -> int:
    l, r = 1, n
    while l < r:
        v = (l + r) // 2
        if isBadVersion(v):
            r = v
        else:
            l = v + 1
    return l



def main():
    assert firstBadVersion(5) == 4
    assert firstBadVersion(4) == 4

if __name__ == "__main__":
    main()
