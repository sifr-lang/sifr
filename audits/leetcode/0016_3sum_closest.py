from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 16: 3Sum Closest
# Python version

def threeSumClosest(nums: List[int], target: int) -> int:

    nums.sort()
    best = float('inf')
    
    for i in range(len(nums) - 2):
        
        val = nums[i]
        left = i + 1
        right = len(nums) - 1
        
        while left < right:
            
            currentGap = abs(target - (val + nums[left] + nums[right]))
            
            if abs(best - target) > currentGap:
                best = val + nums[left] + nums[right]
            if val + nums[left] + nums[right] < target:
                left += 1
            elif val + nums[left] + nums[right] > target:
                right -= 1
            else: #closest it can get 
                return target                
            
    return best



def main():
    assert threeSumClosest([-1,2,1,-4], 1) == 2

if __name__ == "__main__":
    main()
