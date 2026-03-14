from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 280: Wiggle Sort
# Python version

def wiggleSort(nums: List[int]) -> None:
    for i in range(1, len(nums)):
        if (i % 2 == 1 and nums[i] < nums[i - 1]) or (i % 2 == 0 and nums[i] > nums[i - 1]):
            nums[i], nums[i - 1] = nums[i - 1], nums[i]



def main():
    arg0 = [3, 5, 2, 1, 6, 4]
    _result = wiggleSort(arg0)
    assert arg0 == [3, 5, 1, 6, 2, 4]
    arg0 = [6, 6, 5, 6, 3, 8]
    _result = wiggleSort(arg0)
    assert arg0 == [6, 6, 5, 6, 3, 8]

if __name__ == "__main__":
    main()
