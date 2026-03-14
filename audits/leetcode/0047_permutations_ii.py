from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 47: Permutations Ii
# Python version



def permuteUnique(nums: List[int]) -> List[List[int]]:
    result = []
    counter = collections.Counter(nums)

    def backtrack(perm, counter):
        if len(perm) == len(nums):
            result.append(perm.copy())

        for n in counter:
            if counter[n] == 0:
                continue
            perm.append(n)
            counter[n] -= 1
            backtrack(perm, counter)
            perm.pop()
            counter[n] += 1

    backtrack([], counter)

    return result



def main():
    assert permuteUnique([1,1,2]) == [[1, 1, 2], [1, 2, 1], [2, 1, 1]]

if __name__ == "__main__":
    main()
