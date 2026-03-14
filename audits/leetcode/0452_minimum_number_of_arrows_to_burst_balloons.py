from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 452: Minimum Number Of Arrows To Burst Balloons
# Python version

def findMinArrowShots(points: List[List[int]]) -> int:
    points.sort()

    res = len(points)
    prev = points[0]
    for i in range(1, len(points)):
        curr = points[i]
        if curr[0] <= prev[1]:
            res -= 1
            prev = [curr[0], min(curr[1], prev[1])]
        else:
            prev = curr
        
    return res



def main():
    assert findMinArrowShots([[10,16],[2,8],[1,6],[7,12]]) == 2

if __name__ == "__main__":
    main()
