from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 665: Non Decreasing Array
# Python version

def checkPossibility(nums):
    if len(nums) <= 2:
        return True
    changed = False
    for i, num in enumerate(nums):
        if i == len(nums) - 1 or num <= nums[i + 1]:
            continue
        if changed:
            return False
        if i == 0 or nums[i + 1] >= nums[i - 1]:
            nums[i] = nums[i + 1]
        else:
            nums[i + 1] = nums[i]
        changed = True
    return True



def main():
    assert checkPossibility([4,2,3]) == True
    assert checkPossibility([4,2,1]) == False

if __name__ == "__main__":
    main()
