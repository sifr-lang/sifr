from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 416: Partition Equal Subset Sum
# Python version

def canPartition(nums: List[int]) -> bool:
    if sum(nums) % 2:
        return False

    dp = set()
    dp.add(0)
    target = sum(nums) // 2

    for i in range(len(nums) - 1, -1, -1):
        nextDP = set()
        for t in dp:
            if (t + nums[i]) == target:
                return True
            nextDP.add(t + nums[i])
            nextDP.add(t)
        dp = nextDP
    return False



def main():
    assert canPartition([1,5,11,5]) == True
    assert canPartition([1,2,3,5]) == False

if __name__ == "__main__":
    main()
