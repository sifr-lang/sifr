from __future__ import annotations
import collections
import heapq
import math
import random
from collections import Counter, defaultdict, deque
from functools import cache, cmp_to_key, lru_cache
from math import ceil, sqrt

# LeetCode 168: Excel Sheet Column Title
# Python version

def convertToTitle(columnNumber: int) -> str:
    # Time: O(logn) - Log base 26 of n
    res = ""
    while columnNumber > 0:
        remainder = (columnNumber - 1) % 26
        res += chr(ord('A') + remainder)
        columnNumber = (columnNumber - 1) // 26

    return res[::-1] # reverse output



def main():
    assert convertToTitle(1) == 'A'
    assert convertToTitle(28) == 'AB'
    assert convertToTitle(701) == 'ZY'

if __name__ == "__main__":
    main()
