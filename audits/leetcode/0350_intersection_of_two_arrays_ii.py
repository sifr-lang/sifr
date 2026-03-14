from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 350: Intersection Of Two Arrays Ii
# Python version

def intersect(nums1: List[int], nums2: List[int]) -> List[int]:
    counter1 = Counter(nums1)
    counter2 = Counter(nums2)

    # Using defaultdict to handle missing keys more efficiently
    counter1 = defaultdict(int, counter1)
    counter2 = defaultdict(int, counter2)

    intersection = []

    for num, freq in counter1.items():
        min_freq = min(freq, counter2[num])
        if min_freq > 0:
            intersection.extend([num] * min_freq)

    return intersection



def main():
    assert intersect([1, 2, 2, 1], [2, 2]) == [2, 2]
    assert intersect([4, 9, 5], [9, 4, 9, 8, 4]) == [4, 9]

if __name__ == "__main__":
    main()
