from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 502: Ipo
# Python version

def findMaximizedCapital(k: int, w: int, profits: List[int], capital: List[int]) -> int:
    # O(nlogn)
    maxProfit = [] # only projects we can afford
    minCapital = [(c, p) for c, p in zip(capital, profits)]
    heapq.heapify(minCapital)

    for i in range(k):
        
        while minCapital and minCapital[0][0] <= w:
            c, p = heapq.heappop(minCapital)
            heapq.heappush(maxProfit, -1 * p)
        if not maxProfit:
            break
        w += -1 * heapq.heappop(maxProfit)
    return w



def main():
    assert findMaximizedCapital(2, 0, [1, 2, 3], [0, 1, 1]) == 4
    assert findMaximizedCapital(3, 0, [1, 2, 3], [0, 1, 2]) == 6

if __name__ == "__main__":
    main()
