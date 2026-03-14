from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 2001: Number Of Pairs Of Interchangeable Rectangles
# Python version

def interchangeableRectangles(rectangles: List[List[int]]) -> int:
    count = {}  # { W / H : Count }
    res = 0

    for w, h in rectangles:
        # Increment the count for the ratio
        count[w / h] = 1 + count.get(w / h, 0)

    for c in count.values():
        res += (c * (c - 1)) // 2

    return res



def main():
    assert interchangeableRectangles([[4, 8], [3, 6], [10, 20], [15, 30]]) == 6
    assert interchangeableRectangles([[4, 5], [7, 8]]) == 0

if __name__ == "__main__":
    main()
