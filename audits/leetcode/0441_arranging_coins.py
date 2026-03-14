from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 441: Arranging Coins
# Python version

def arrangeCoins(n: int) -> int:
    l, r = 1, n
    res = 0
    while l <=r:
        mid = (l+r)//2
        coins = (mid /2) * (mid+1)
        if coins > n:
            r = mid - 1
        else:
            l = mid + 1
            res = max(mid, res)
    return res



def main():
    assert arrangeCoins(5) == 2
    assert arrangeCoins(8) == 3

if __name__ == "__main__":
    main()
