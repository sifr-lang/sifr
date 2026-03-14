from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 703: Kth Largest Element In A Stream
# Python version

class KthLargest:
    def __init__(self, k: int, nums: List[int]):
        self.minHeap, self.k = nums, k
        heapq.heapify(self.minHeap)
        while len(self.minHeap) > k:
            heapq.heappop(self.minHeap)
    def add(self, val: int) -> int:
        heapq.heappush(self.minHeap, val)
        if len(self.minHeap) > self.k:
            heapq.heappop(self.minHeap)
        return self.minHeap[0]

def main():
    obj = KthLargest(3, [4, 5, 8, 2])
    assert obj.add(3) == 4
    assert obj.add(5) == 5
    assert obj.add(10) == 5
    assert obj.add(9) == 8
    assert obj.add(4) == 8

if __name__ == "__main__":
    main()
