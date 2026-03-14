from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 69: Sqrtx
# Python version

def mySqrt(x: int) -> int:
    l, r = 0, x
    while l <= r:
        mid = (l + r) // 2
        if mid * mid == x:
            return mid
        if mid * mid < x:
            l = mid + 1
        else:
            r = mid - 1
    return r



def main():
    assert mySqrt(4) == 2
    assert mySqrt(8) == 2
    assert mySqrt(0) == 0

if __name__ == "__main__":
    main()
