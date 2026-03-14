from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 1929: Concatenation of Array
# Python version with test cases

def getConcatenation(nums: list[int]) -> list[int]:
    ans: list[int] = []
    for i in range(2):
        for j in range(len(nums)):
            ans.append(nums[j])
    return ans

def main():
    assert getConcatenation([1, 2, 1]) == [1, 2, 1, 1, 2, 1]
    assert getConcatenation([1, 3, 2, 1]) == [1, 3, 2, 1, 1, 3, 2, 1]

if __name__ == "__main__":
    main()
