from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 1383: Maximum Performance Of A Team
# Python version

def maxPerformance(n: int, speed: List[int], efficiency: List[int], k: int) -> int:
    mod = 10 ** 9 + 7
    eng = []
    for eff, spd in zip(efficiency, speed):
        eng.append([eff, spd])
    eng.sort(reverse = True)
    
    res, speed = 0, 0
    minHeap = []
    
    for eff, spd in eng:
        if len(minHeap) == k:
            speed -= heapq.heappop(minHeap)
        speed += spd
        heapq.heappush(minHeap, spd)
        res = max(res, eff * speed)
    return res % mod



def main():
    assert maxPerformance(6, [2, 10, 3, 1, 5, 8], [5, 4, 3, 9, 7, 2], 2) == 60
    assert maxPerformance(6, [2, 10, 3, 1, 5, 8], [5, 4, 3, 9, 7, 2], 3) == 68
    assert maxPerformance(6, [2, 10, 3, 1, 5, 8], [5, 4, 3, 9, 7, 2], 4) == 72

if __name__ == "__main__":
    main()
