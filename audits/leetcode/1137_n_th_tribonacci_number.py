from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 1137: N Th Tribonacci Number
# Python version

Memo = {}


def tribonacci(n: int):
    if n in Memo:
        return Memo[n]
    if n == 0:
        return 0
    if n == 1:
        return 1
    if n == 2:
        return 1
    Memo[n] = (
        tribonacci(n - 1) + tribonacci(n - 2) + tribonacci(n - 3)
    )
    return Memo[n]



def main():
    assert tribonacci(4) == 4
    assert tribonacci(25) == 1389537

if __name__ == "__main__":
    main()
