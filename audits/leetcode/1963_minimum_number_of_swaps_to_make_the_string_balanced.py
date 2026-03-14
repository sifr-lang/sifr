from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 1963: Minimum Number Of Swaps To Make The String Balanced
# Python version

def minSwaps(s: str) -> int:
    extraClose, maxClose = 0, 0

    for c in s:
        if c == "[":
            extraClose -= 1
        else:
            extraClose += 1

        maxClose = max(maxClose, extraClose)

    return (maxClose + 1) // 2  # Or math.ceil(maxClose / 2)



def main():
    assert minSwaps("][][") == 1
    assert minSwaps("[][][]") == 0

if __name__ == "__main__":
    main()
