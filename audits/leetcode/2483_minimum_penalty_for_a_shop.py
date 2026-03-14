from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 2483: Minimum Penalty For A Shop
# Python version

def bestClosingTime(customers: str) -> int:
    
    curPenalty = res = minPenalty = 0

    for i, ele in enumerate(customers):
        if ele == 'Y':
            curPenalty -= 1
            if curPenalty < minPenalty:
                res = i+1
                curPenalty = minPenalty
        else:
            curPenalty += 1

    return res



def main():
    assert bestClosingTime("YYNY") == 2
    assert bestClosingTime("NNNNN") == 0

if __name__ == "__main__":
    main()
