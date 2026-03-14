from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 560: Subarray Sum Equals K
# Python version

def subarraySum(nums: List[int], k: int) -> int:
    count = 0
    sum = 0
    dic = {}
    dic[0] = 1
    for i in range(len(nums)):
        sum += nums[i]
        if sum-k in dic:
            count += dic[sum-k]
        dic[sum] = dic.get(sum, 0)+1
    return count

# Time Complexity :
#     O(N) -> Where N is the size of the array and we are iterating over the array once

# Space Complexity:
#     O(N) -> Creating a hashmap/dictionary



def main():
    assert subarraySum([1,1,1], 2) == 2
    assert subarraySum([1,2,3], 3) == 2

if __name__ == "__main__":
    main()
