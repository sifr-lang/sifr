from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 862: Shortest Subarray With Sum At Least K
# Python version

def shortestSubarray(nums: List[int], k: int) -> int:
    size = len(nums)
    pre = [0]
    for i in nums:
        pre.append(pre[-1] + i)

    ans = size + 1
    monoq = collections.deque()
    for i, val in enumerate(pre):
        while monoq and val <= pre[monoq[-1]]:
            monoq.pop()
        while monoq and val - pre[monoq[0]] >= k:
            ans = min(ans, i - monoq.popleft())
        
        monoq.append(i)
    
    return ans if ans < size + 1 else -1   


def main():
    assert shortestSubarray([1], 1) == 1
    assert shortestSubarray([1, 2], 4) == -1
    assert shortestSubarray([2, -1, 2], 3) == 3

if __name__ == "__main__":
    main()
