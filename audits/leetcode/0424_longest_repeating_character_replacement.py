from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 424: Longest Repeating Character Replacement
# Python version

def characterReplacement(s: str, k: int) -> int:
    count = {}
    
    l = 0
    maxf = 0
    for r in range(len(s)):
        count[s[r]] = 1 + count.get(s[r], 0)
        maxf = max(maxf, count[s[r]])

        if (r - l + 1) - maxf > k:
            count[s[l]] -= 1
            l += 1

    return (r - l + 1)



def main():
    assert characterReplacement("ABAB", 2) == 4
    assert characterReplacement("AABABBA", 1) == 4

if __name__ == "__main__":
    main()
