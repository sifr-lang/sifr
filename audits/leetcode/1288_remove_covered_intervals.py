from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 1288: Remove Covered Intervals
# Python version

def removeCoveredIntervals(intervals: List[List[int]]) -> int:
    # sort on the basis of inc li first and then on the basis of dec length (=> -ri)
    intervals.sort(key=lambda x: (x[0], -x[1]))
    
    covered, maxri = 0, 0
    
    for _, ri in intervals:
        if ri > maxri:
            maxri = ri
        else:
            covered += 1
            
    return len(intervals) - covered


def main():
    assert removeCoveredIntervals([[1,4],[3,6],[2,8]]) == 2

if __name__ == "__main__":
    main()
