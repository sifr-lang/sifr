from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 231: Power Of Two
# Python version

def isPowerOfTwo(n: int) -> bool:
    x = 1
    while x < n:
        x *= 2
    return x == n        

# Bit manipulation

def isPowerOfTwo(n: int) -> bool:
    return n > 0 and (n & (n - 1)) == 0

# Bit manipulation

def isPowerOfTwo(n: int) -> bool:
    return n > 0 and ((1 << 30) % n) == 0



def main():
    assert isPowerOfTwo(1) == True
    assert isPowerOfTwo(16) == True
    assert isPowerOfTwo(3) == False

if __name__ == "__main__":
    main()
