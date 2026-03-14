from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 190: Reverse Bits
# Python version

def reverseBits(n: int) -> int:
    res = 0
    for i in range(32):
        bit = (n >> i) & 1
        res += (bit << (31 - i))
    return res



def main():
    assert reverseBits(43261596) == 964176192
    assert reverseBits(4294967293) == 3221225471

if __name__ == "__main__":
    main()
