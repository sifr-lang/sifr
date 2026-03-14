from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 338: Counting Bits
# Python version

def countBits(n: int) -> List[int]:
    dp = [0] * (n + 1)
    offset = 1

    for i in range(1, n + 1):
        if offset * 2 == i:
            offset = i
        dp[i] = 1 + dp[i - offset]
    return dp

# Another dp solution

def countBits(n: int) -> List[int]:
    res = [0] * (n + 1)
    for i in range(1, n + 1):
        if i % 2 == 1:
            res[i] = res[i - 1] + 1
        else:
            res[i] = res[i // 2]
    return res
# This solution is based on the division of odd and even numbers. 
# I think it's easier to understand.
# This is my full solution, covering the details: https://leetcode.com/problems/counting-bits/solutions/4411054/odd-and-even-numbers-a-easier-to-understanding-way-of-dp/



def main():
    assert countBits(2) == [0, 1, 1]
    assert countBits(5) == [0, 1, 1, 2, 1, 2]

if __name__ == "__main__":
    main()
