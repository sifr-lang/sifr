from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 66: Plus One
# Python version

def plusOne(digits: List[int]) -> List[int]:
    one = 1
    i = 0
    digits = digits[::-1]

    while one:
        if i < len(digits):
            if digits[i] == 9:
                digits[i] = 0
            else:
                digits[i] += 1
                one = 0
        else:
            digits.append(one)
            one = 0
        i += 1
    return digits[::-1]



def main():
    assert plusOne([1,2,3]) == [1, 2, 4]
    assert plusOne([9]) == [1, 0]

if __name__ == "__main__":
    main()
