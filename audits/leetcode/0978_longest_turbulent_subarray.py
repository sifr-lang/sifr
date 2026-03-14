from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 978: Longest Turbulent Subarray
# Python version

def maxTurbulenceSize(arr: List[int]) -> int:
    l, r = 0, 1
    res, prev = 1, ""

    while r < len(arr):
        if arr[r - 1] > arr[r] and prev != ">":
            res = max(res, r - l + 1)
            r += 1
            prev = ">"
        elif arr[r - 1] < arr[r] and prev != "<":
            res = max(res, r - l + 1)
            r += 1
            prev = "<"
        else:
            r = r + 1 if arr[r] == arr[r - 1] else r
            l = r - 1
            prev = ""
    return res



def main():
    assert maxTurbulenceSize([9,4,2,10,7,8,8,1,9]) == 5
    assert maxTurbulenceSize([100]) == 1

if __name__ == "__main__":
    main()
