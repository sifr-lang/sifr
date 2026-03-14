from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 1985: Find The Kth Largest Integer In The Array
# Python version

def kthLargestNumber(nums: List[str], k: int) -> str:
    maxHeap = [-int(n) for n in nums]
    heapq.heapify(maxHeap)
    while k>1:
        heapq.heappop(maxHeap)
        k-=1
    return str(-maxHeap[0])



def main():
    assert kthLargestNumber(['3', '6', '7', '10'], 4) == '3'
    assert kthLargestNumber(['2', '21', '12', '1'], 3) == '2'
    assert kthLargestNumber(['0', '0'], 2) == '0'

if __name__ == "__main__":
    main()
