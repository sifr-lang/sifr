from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 496: Next Greater Element I
# Python version

def nextGreaterElement(nums1: List[int], nums2: List[int]) -> List[int]:

    # O (n + m)
    nums1Idx = { n:i for i, n in enumerate(nums1) }
    res = [-1] * len(nums1)

    stack = []
    for i in range(len(nums2)):
        cur = nums2[i]

        # while stack exists and current is greater than the top of the stack
        while stack and cur > stack[-1]:
            val = stack.pop() # take top val
            idx = nums1Idx[val]
            res[idx] = cur

        if cur in nums1Idx:
            stack.append(cur)
    
    return res


    # O (n * m)
    nums1Idx = { n:i for i, n in enumerate(nums1) }
    res = [-1] * len(nums1)
    
    for i in range(len(nums2)):
        if nums2[i] not in nums1Idx:
            continue
        for j in range(i + 1, len(nums2)):
            if nums2[j] > nums2[i]:
                idx = nums1Idx[nums2[i]]
                res[idx] = nums2[j]
                break
    
    return res


def main():
    assert nextGreaterElement([4,1,2], [1,3,4,2]) == [-1, 3, -1]

if __name__ == "__main__":
    main()
