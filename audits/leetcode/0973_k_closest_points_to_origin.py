from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 973: K Closest Points To Origin
# Python version

def kClosest(points: List[List[int]], k: int) -> List[List[int]]:
    minHeap = []
    for x, y in points:
        dist = (x ** 2) + (y ** 2)
        minHeap.append((dist, x, y))
    
    heapq.heapify(minHeap)
    res = []
    for _ in range(k):
        _, x, y = heapq.heappop(minHeap)
        res.append((x, y))
    return res


def main():
    assert kClosest([[1, 3], [-2, 2]], 1) == [(-2, 2)]
    assert kClosest([[3, 3], [5, -1], [-2, 4]], 2) == [(3, 3), (-2, 4)]

if __name__ == "__main__":
    main()
