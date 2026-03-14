from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 1456: Maximum Number Of Vowels In A Substring Of Given Length
# Python version

def maxVowels(s: str, k: int) -> int:
    l, res, total = 0, 0, 0
    vowels = "aeiou"

    for r in range(len(s)):
        if s[r] in vowels:
            total += 1
        if (r - l + 1) > k:
            if s[l] in vowels:
                total -= 1
            l += 1
        res = max(res, total)
    return res

        
        



def main():
    assert maxVowels("abciiidef", 3) == 3
    assert maxVowels("aeiou", 2) == 2

if __name__ == "__main__":
    main()
