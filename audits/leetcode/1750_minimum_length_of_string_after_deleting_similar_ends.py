from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 1750: Minimum Length Of String After Deleting Similar Ends
# Python version

def minimumLength(s: str) -> int:
    l, r = 0, len(s) - 1

    while l < r and s[l] == s[r]:
        tmp = s[l]
        while l <= r and s[l] == tmp:
            l += 1
        while l <= r and s[r] == tmp:
            r -= 1
    return (r - l + 1)



def main():
    assert minimumLength("ca") == 2
    assert minimumLength("cabaabac") == 0

if __name__ == "__main__":
    main()
