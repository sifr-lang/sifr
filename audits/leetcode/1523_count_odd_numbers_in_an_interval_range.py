from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 1523: Count Odd Numbers In An Interval Range
# Python version

def countOdds(low: int, high: int) -> int:
    if low%2!=0 or high%2!=0:
        return (high-low)//2 +1
    return (high-low)//2



def main():
    assert countOdds(3, 7) == 3
    assert countOdds(8, 10) == 1

if __name__ == "__main__":
    main()
